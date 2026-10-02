use rustls::{ServerConfig, ServerConnection, StreamOwned, pki_types::PrivatePkcs8KeyDer};
use rvvdk_vsphere::{
    Cleanup, ConnectionPolicy, ConnectionReuse, Credentials, Error, InventoryLimits, Timeouts,
    discover,
};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

#[derive(Clone)]
struct Reply {
    status: u16,
    body: String,
    binary: Option<Vec<u8>>,
    headers: String,
    delay: Duration,
    declared: Option<usize>,
    chunked: bool,
}
impl Reply {
    fn soap(method: &str, body: &str) -> Self {
        Self {
            status: 200,
            body: envelope(&format!(
                "<{method}Response xmlns='urn:vim25'>{body}</{method}Response>"
            )),
            binary: None,
            headers: String::new(),
            delay: Duration::ZERO,
            declared: None,
            chunked: false,
        }
    }
    fn fault(kind: &str) -> Self {
        Self {
            status: 500,
            body: envelope(&format!(
                "<s:Fault><faultstring>PRIVATE fault text</faultstring><detail><{kind} xmlns='urn:vim25'/></detail></s:Fault>"
            )),
            binary: None,
            headers: String::new(),
            delay: Duration::ZERO,
            declared: None,
            chunked: false,
        }
    }
}
fn envelope(body: &str) -> String {
    format!(
        "<s:Envelope xmlns:s='http://schemas.xmlsoap.org/soap/envelope/' xmlns:xsi='http://www.w3.org/2001/XMLSchema-instance'><s:Body>{body}</s:Body></s:Envelope>"
    )
}
fn service() -> Reply {
    Reply::soap(
        "RetrieveServiceContent",
        "<returnval><about><version>8.0.3</version><build>24677879</build><apiVersion>8.0.3.0</apiVersion><apiType>HostAgent</apiType></about><sessionManager type='SessionManager'>SessionManager</sessionManager><propertyCollector type='PropertyCollector'>propertyCollector</propertyCollector><rootFolder type='Folder'>root</rootFolder><licenseManager type='LicenseManager'>license</licenseManager></returnval>",
    )
}
fn login() -> Reply {
    let mut reply = Reply::soap("Login", "<returnval/>");
    reply.headers =
        "Set-Cookie: vmware_soap_session=\"PRIVATE-session\"; Path=/sdk; Secure; HttpOnly\r\n"
            .to_owned();
    reply
}
fn logout() -> Reply {
    Reply::soap("Logout", "")
}
fn properties(kind: &str, id: &str, props: &str) -> Reply {
    Reply::soap(
        "RetrievePropertiesEx",
        &format!("<returnval><objects><obj type='{kind}'>{id}</obj>{props}</objects></returnval>"),
    )
}
fn property(name: &str, value: &str) -> String {
    let array_type = if name == "childEntity" {
        " xsi:type='ArrayOfManagedObjectReference'"
    } else {
        ""
    };
    format!("<propSet><name>{name}</name><val{array_type}>{value}</val></propSet>")
}
fn license() -> Reply {
    properties(
        "LicenseManager",
        "license",
        &property(
            "licenses",
            "<LicenseInfo><licenseKey>PRIVATE-license</licenseKey><editionKey>esx.hypervisor.cpuPackageCoreLimited</editionKey></LicenseInfo>",
        ),
    )
}
fn root() -> Reply {
    properties("Folder", "root", &property("childEntity", ""))
}
fn minimal() -> Vec<Reply> {
    vec![service(), login(), license(), root(), logout()]
}
fn full() -> Vec<Reply> {
    let root = properties(
        "Folder",
        "root",
        &property(
            "childEntity",
            "<ManagedObjectReference type='HostSystem'>host</ManagedObjectReference><ManagedObjectReference type='Datastore'>store</ManagedObjectReference><ManagedObjectReference xsi:type='ManagedObjectReference' type='VirtualMachine'>vm-private</ManagedObjectReference>",
        ),
    );
    let host = properties(
        "HostSystem",
        "host",
        &(property(
            "summary.hardware",
            "<numCpuThreads>10</numCpuThreads><memorySize>33545613312</memorySize>",
        ) + &property("runtime.connectionState", "connected")),
    );
    let store = properties(
        "Datastore",
        "store",
        &(property(
            "summary",
            "<type>VMFS</type><capacity>1099243192320</capacity><freeSpace>994927706112</freeSpace><accessible>true</accessible>",
        ) + &property("info", "<vmfs><version>6.82</version></vmfs>")),
    );
    let vm=properties("VirtualMachine","vm-private",&[
        property("config.uuid","01234567-89ab-cdef-0123-456789abcdef"),property("runtime.powerState","poweredOn"),property("config.guestId","fedora64Guest"),property("config.template","false"),property("guest.toolsRunningStatus","guestToolsRunning"),property("disabledMethod","<string>ExportVm</string>"),
        property("config.hardware.device","<VirtualDevice xsi:type='VirtualDisk'><key>2000</key><capacityInKB>31457280</capacityInKB><capacityInBytes>32212254720</capacityInBytes><backing xsi:type='VirtualDiskFlatVer2BackingInfo'><fileName>PRIVATE-path</fileName><diskMode>persistent</diskMode><thinProvisioned>false</thinProvisioned></backing></VirtualDevice>")
    ].concat());
    vec![
        service(),
        login(),
        license(),
        root,
        host,
        store,
        vm,
        logout(),
    ]
}
type RequestObserver = Arc<dyn Fn(&str) + Send + Sync>;
struct Server {
    endpoint: String,
    pin: String,
    #[allow(dead_code)]
    progress_failure: Arc<AtomicBool>,
    requests: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Server {
    fn start(replies: Vec<Reply>) -> Self {
        Self::start_with_nodelay(replies, false)
    }
    fn start_with_nodelay(replies: Vec<Reply>, nodelay: bool) -> Self {
        Self::start_observed(replies, nodelay, None)
    }
    fn start_observed(
        replies: Vec<Reply>,
        nodelay: bool,
        observer: Option<RequestObserver>,
    ) -> Self {
        Self::start_with_idle(replies, nodelay, observer, Duration::from_millis(300))
    }
    fn start_with_idle(
        replies: Vec<Reply>,
        nodelay: bool,
        observer: Option<RequestObserver>,
        idle: Duration,
    ) -> Self {
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_owned()]).unwrap();
        let pin = Sha256::digest(cert.cert.der())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let legacy_pin = sha1::Sha1::digest(cert.cert.der())
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(":");
        let config =
            ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                .with_safe_default_protocol_versions()
                .unwrap()
                .with_no_client_auth()
                .with_single_cert(
                    vec![cert.cert.der().clone()],
                    PrivatePkcs8KeyDer::from(cert.signing_key.serialize_der()).into(),
                )
                .unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("https://{}", listener.local_addr().unwrap());
        let replies = replies
            .into_iter()
            .map(|mut r| {
                r.body = r
                    .body
                    .replace("ENDPOINT", &endpoint)
                    .replace("WILDCARD", &endpoint.replacen("127.0.0.1", "*", 1))
                    .replace("LEGACY_PIN", &legacy_pin)
                    .replace("THUMBPRINT", &pin);
                r
            })
            .collect::<Vec<_>>();
        listener.set_nonblocking(true).unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let observed = requests.clone();
        let progress_failure = Arc::new(AtomicBool::new(false));
        let fail_progress = progress_failure.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let halted = stop.clone();
        let worker = thread::spawn(move || {
            let config = Arc::new(config);
            let replies = Arc::new(Mutex::new(std::collections::VecDeque::from(replies)));
            let mut connections = Vec::new();
            while !halted.load(Ordering::Relaxed) && !replies.lock().unwrap().is_empty() {
                let (stream, _) = match listener.accept() {
                    Ok(v) => v,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2));
                        continue;
                    }
                    Err(e) => panic!("{e}"),
                };
                stream.set_nodelay(nodelay).unwrap();
                stream.set_read_timeout(Some(idle)).unwrap();
                stream
                    .set_write_timeout(Some(Duration::from_millis(300)))
                    .unwrap();
                let config = config.clone();
                let halted = halted.clone();
                let replies = replies.clone();
                let observed = observed.clone();
                let observer = observer.clone();
                let fail_progress = fail_progress.clone();
                connections.push(thread::spawn(move || {
                let mut stream =
                    StreamOwned::new(ServerConnection::new(config.clone()).unwrap(), stream);
                while !halted.load(Ordering::Relaxed) && !replies.lock().unwrap().is_empty() {
                    let mut headers = Vec::new();
                    while !headers.ends_with(b"\r\n\r\n") && headers.len() < 16384 {
                        let mut b = [0];
                        if stream.read_exact(&mut b).is_err() {
                            break;
                        }
                        headers.push(b[0]);
                    }
                    if !headers.ends_with(b"\r\n\r\n") {
                        break;
                    }
                    let headers = String::from_utf8(headers).unwrap();
                    let length: usize = headers
                        .lines()
                        .find_map(|l| {
                            l.to_ascii_lowercase()
                                .strip_prefix("content-length: ")
                                .map(str::to_owned)
                        })
                        .unwrap_or_else(|| "0".to_owned())
                        .parse()
                        .unwrap();
                    assert!(length < 65536);
                    let mut body = vec![0; length];
                    if stream.read_exact(&mut body).is_err() {
                        break;
                    }
                    let body = String::from_utf8(body).unwrap();
                    observed.lock().unwrap().push(format!("{headers}{body}"));
                    if let Some(observer) = &observer { observer(&body); }
                    let reply = if body.contains("<HttpNfcLeaseProgress ") {
                        if fail_progress.load(Ordering::Relaxed) { Reply::fault("RuntimeFault") }
                        else { Reply::soap("HttpNfcLeaseProgress", "") }
                    } else {
                        replies.lock().unwrap().pop_front().unwrap()
                    };
                    thread::sleep(reply.delay);
                    let bytes = reply.binary.as_deref().unwrap_or(reply.body.as_bytes());
                    let mut out = if reply.chunked {
                        format!("HTTP/1.1 {} status\r\nContent-Type: text/xml\r\nTransfer-Encoding: chunked\r\n{}\r\n", reply.status, reply.headers).into_bytes()
                    } else {
                        format!("HTTP/1.1 {} status\r\nContent-Type: text/xml\r\nContent-Length: {}\r\n{}\r\n", reply.status, reply.declared.unwrap_or(bytes.len()), reply.headers).into_bytes()
                    };
                    if reply.chunked {
                        for chunk in bytes.chunks(8192) {
                            out.extend_from_slice(format!("{:x}\r\n", chunk.len()).as_bytes());
                            out.extend_from_slice(chunk);
                            out.extend_from_slice(b"\r\n");
                        }
                        out.extend_from_slice(b"0\r\n\r\n");
                    } else {
                        out.extend_from_slice(bytes);
                    }
                    if stream
                        .write_all(&out)
                        .and_then(|_| stream.flush())
                        .is_err()
                    {
                        break;
                    }
                    if headers.to_ascii_lowercase().contains("connection: close") {
                        break;
                    }
                }
                }));
            }
            for connection in connections {
                connection.join().unwrap();
            }
        });
        Self {
            endpoint,
            pin,
            progress_failure,
            requests,
            stop,
            worker: Some(worker),
        }
    }
    fn policy(&self) -> ConnectionPolicy {
        ConnectionPolicy::pinned(&self.endpoint, &self.pin).unwrap()
    }
    fn requests(&self) -> Vec<String> {
        self.requests.lock().unwrap().clone()
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.worker.take().unwrap().join().unwrap();
    }
}
fn credentials() -> Credentials {
    Credentials::new("private-user<&".to_owned(), "PRIVATE-password<&".to_owned()).unwrap()
}
