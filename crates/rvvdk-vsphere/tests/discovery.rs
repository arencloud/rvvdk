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
struct Server {
    endpoint: String,
    pin: String,
    requests: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Server {
    fn start(replies: Vec<Reply>) -> Self {
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
                stream
                    .set_read_timeout(Some(Duration::from_millis(300)))
                    .unwrap();
                stream
                    .set_write_timeout(Some(Duration::from_millis(300)))
                    .unwrap();
                let config = config.clone();
                let halted = halted.clone();
                let replies = replies.clone();
                let observed = observed.clone();
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
                    let reply = if body.contains("<HttpNfcLeaseProgress ") {
                        Reply::soap("HttpNfcLeaseProgress", "")
                    } else {
                        replies.lock().unwrap().pop_front().unwrap()
                    };
                    thread::sleep(reply.delay);
                    let out = if reply.chunked {
                        let mut encoded = String::new();
                        for chunk in reply.body.as_bytes().chunks(8192) {
                            encoded.push_str(&format!(
                                "{:x}\r\n{}\r\n",
                                chunk.len(),
                                std::str::from_utf8(chunk).unwrap()
                            ));
                        }
                        format!(
                            "HTTP/1.1 {} status\r\nContent-Type: text/xml\r\nTransfer-Encoding: chunked\r\n{}\r\n{}0\r\n\r\n",
                            reply.status, reply.headers, encoded
                        )
                    } else {
                        format!(
                            "HTTP/1.1 {} status\r\nContent-Type: text/xml\r\nContent-Length: {}\r\n{}\r\n{}",
                            reply.status,
                            reply.declared.unwrap_or(reply.body.len()),
                            reply.headers,
                            reply.body
                        )
                    };
                    if stream
                        .write_all(out.as_bytes())
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

#[test]
fn scoped_operation_futures_remain_send() {
    fn require_send(_: impl Send) {}
    let policy = ConnectionPolicy::pinned("https://example.invalid", &"a".repeat(64)).unwrap();
    let credentials = credentials();
    require_send(discover(
        policy.clone(),
        &credentials,
        InventoryLimits::default(),
    ));
    require_send(rvvdk_vsphere::export_vm(
        policy,
        &credentials,
        rvvdk_vsphere::ExportOptions::probe(1024),
    ));
}

#[tokio::test]
async fn tls_inventory_identity_redaction_and_reuse() {
    let server = Server::start(full());
    let report = discover(server.policy(), &credentials(), InventoryLimits::default())
        .await
        .unwrap();
    assert!(report.is_success(), "{report:?}");
    assert_eq!(report.certificate_checks, 1);
    let inventory = report.inventory.as_ref().unwrap();
    assert_eq!(
        inventory.vms[0].disks[0].capacity_bytes,
        30 * 1024 * 1024 * 1024
    );
    assert_eq!(inventory.vms[0].identity.managed_reference(), "vm-private");
    assert_eq!(
        inventory.vms[0].identity.bios_uuid(),
        "01234567-89ab-cdef-0123-456789abcdef"
    );
    let json = serde_json::to_string(&report).unwrap();
    for private in [
        "PRIVATE",
        "vm-private",
        "01234567-89ab-cdef-0123-456789abcdef",
        &server.endpoint,
    ] {
        assert!(!json.contains(private));
        assert!(!format!("{report:?}").contains(private));
    }
    let requests = server.requests();
    assert_eq!(requests.len(), 8);
    assert!(requests[1].contains("PRIVATE-password&lt;&amp;"));
    assert!(!requests[0].to_ascii_lowercase().contains("cookie:"));
    assert!(requests[2..].iter().all(|r| {
        r.to_ascii_lowercase()
            .contains("cookie: vmware_soap_session=\"private-session\"")
    }));
}
#[tokio::test]
async fn fresh_policy_rechecks_pin_for_each_request() {
    let server = Server::start(minimal());
    let report = discover(
        server
            .policy()
            .with_connection_reuse(ConnectionReuse::Fresh),
        &credentials(),
        InventoryLimits::default(),
    )
    .await
    .unwrap();
    assert!(report.is_success(), "{report:?}");
    assert_eq!(report.certificate_checks, 5);
}
#[tokio::test]
async fn wrong_pin_sends_no_http_or_credentials() {
    let server = Server::start(minimal());
    let policy = ConnectionPolicy::pinned(&server.endpoint, &"0".repeat(64)).unwrap();
    let report = discover(policy, &credentials(), InventoryLimits::default())
        .await
        .unwrap();
    assert_eq!(report.primary_error, Some(Error::Transport));
    assert_eq!(report.cleanup, Cleanup::NotNeeded);
    assert!(server.requests().is_empty());
}
#[tokio::test]
async fn redirect_is_never_followed() {
    let mut redirect = service();
    redirect.status = 302;
    redirect.headers = "Location: https://example.invalid/PRIVATE\r\n".to_owned();
    let server = Server::start(vec![redirect]);
    let report = discover(server.policy(), &credentials(), InventoryLimits::default())
        .await
        .unwrap();
    assert_eq!(report.primary_error, Some(Error::Http));
    assert_eq!(server.requests().len(), 1);
}
#[tokio::test]
async fn authentication_and_cleanup_faults_remain_separate_and_redacted() {
    let server = Server::start(vec![
        service(),
        Reply::fault("InvalidLoginFault"),
        Reply::fault("NotAuthenticatedFault"),
    ]);
    let report = discover(server.policy(), &credentials(), InventoryLimits::default())
        .await
        .unwrap();
    assert_eq!(report.primary_error, Some(Error::InvalidLogin));
    assert_eq!(
        report.cleanup,
        Cleanup::Unconfirmed(Error::NotAuthenticated)
    );
    assert!(!report.is_success());
    assert!(!serde_json::to_string(&report).unwrap().contains("PRIVATE"));
}
#[tokio::test]
async fn malformed_login_still_uses_received_cookie_for_logout() {
    let mut invalid = login();
    invalid.body = "<broken>".to_owned();
    let server = Server::start(vec![service(), invalid, logout()]);
    let report = discover(server.policy(), &credentials(), InventoryLimits::default())
        .await
        .unwrap();
    assert_eq!(report.primary_error, Some(Error::Xml));
    assert_eq!(report.cleanup, Cleanup::LoggedOut);
    assert!(server.requests()[2].contains("PRIVATE-session"));
}
#[tokio::test]
async fn missing_cookie_is_not_reported_as_authenticated() {
    let server = Server::start(vec![
        service(),
        Reply::soap("Login", "<returnval/>"),
        logout(),
    ]);
    let report = discover(server.policy(), &credentials(), InventoryLimits::default())
        .await
        .unwrap();
    assert_eq!(report.primary_error, Some(Error::Cookie));
    assert_eq!(report.cleanup, Cleanup::LoggedOut);
}
#[tokio::test]
async fn missing_property_reports_primary_and_logout_failure() {
    let missing = properties(
        "LicenseManager",
        "license",
        "<missingSet><path>licenses</path><fault>PRIVATE</fault></missingSet>",
    );
    let server = Server::start(vec![
        service(),
        login(),
        missing,
        Reply::fault("RuntimeFault"),
    ]);
    let report = discover(server.policy(), &credentials(), InventoryLimits::default())
        .await
        .unwrap();
    assert_eq!(report.primary_error, Some(Error::MissingProperty));
    assert_eq!(report.cleanup, Cleanup::Unconfirmed(Error::SoapFault));
}
#[tokio::test]
async fn pagination_cancels_cursor_and_preserves_cleanup_error() {
    for cancel_fails in [false, true] {
        let page = Reply::soap(
            "RetrievePropertiesEx",
            "<returnval><token>PRIVATE-cursor&lt;&amp;</token></returnval>",
        );
        let cancel = if cancel_fails {
            Reply::fault("RuntimeFault")
        } else {
            Reply::soap("CancelRetrievePropertiesEx", "")
        };
        let server = Server::start(vec![service(), login(), page, cancel, logout()]);
        let report = discover(server.policy(), &credentials(), InventoryLimits::default())
            .await
            .unwrap();
        assert_eq!(report.primary_error, Some(Error::Pagination));
        assert_eq!(report.cleanup, Cleanup::LoggedOut);
        assert_eq!(
            report.pagination_cleanup_error,
            if cancel_fails {
                Some(Error::SoapFault)
            } else {
                None
            }
        );
        assert!(server.requests()[3].contains("<token>PRIVATE-cursor&lt;&amp;</token>"));
    }
}
#[tokio::test]
async fn size_encoding_and_truncated_responses_fail_closed() {
    for (index, expected) in [Error::ResponseLimit, Error::Http, Error::Transport]
        .into_iter()
        .enumerate()
    {
        let mut reply = service();
        match index {
            0 => reply.declared = Some(2 * 1024 * 1024 + 1),
            1 => reply.headers = "Content-Encoding: gzip\r\n".to_owned(),
            _ => reply.declared = Some(reply.body.len() + 100),
        }
        let server = Server::start(vec![reply]);
        let report = discover(server.policy(), &credentials(), InventoryLimits::default())
            .await
            .unwrap();
        assert_eq!(report.primary_error, Some(expected));
        assert_eq!(report.cleanup, Cleanup::NotNeeded);
    }
}
#[tokio::test]
async fn deadlines_expire_and_still_allow_logout() {
    let mut slow = license();
    slow.delay = Duration::from_millis(180);
    let server = Server::start(vec![service(), login(), slow, logout()]);
    let timeouts = Timeouts {
        request: Duration::from_millis(100),
        inactivity: Duration::from_millis(100),
        discovery: Duration::from_secs(2),
        cleanup: Duration::from_secs(2),
        ..Timeouts::default()
    };
    let report = discover(
        server.policy().with_timeouts(timeouts).unwrap(),
        &credentials(),
        InventoryLimits::default(),
    )
    .await
    .unwrap();
    assert_eq!(report.primary_error, Some(Error::Deadline));
    assert_eq!(report.cleanup, Cleanup::LoggedOut);
}
#[tokio::test]
async fn inventory_and_disk_limits_logout_without_partial_success() {
    let mut replies = full();
    replies.drain(4..7);
    let server = Server::start(replies);
    let report = discover(
        server.policy(),
        &credentials(),
        InventoryLimits {
            max_objects: 1,
            max_disks_per_vm: 16,
        },
    )
    .await
    .unwrap();
    assert_eq!(report.primary_error, Some(Error::InventoryLimit));
    assert_eq!(report.cleanup, Cleanup::LoggedOut);
    assert!(report.inventory.is_none());
    let mut replies = full();
    let start = replies[6].body.find("<VirtualDevice").unwrap();
    let end = replies[6].body.find("</VirtualDevice>").unwrap() + "</VirtualDevice>".len();
    let disk = replies[6].body[start..end].to_owned();
    replies[6].body.insert_str(end, &disk);
    let server = Server::start(replies);
    let report = discover(
        server.policy(),
        &credentials(),
        InventoryLimits {
            max_objects: 128,
            max_disks_per_vm: 1,
        },
    )
    .await
    .unwrap();
    assert_eq!(report.primary_error, Some(Error::InventoryLimit));
    assert_eq!(report.cleanup, Cleanup::LoggedOut);
}
#[tokio::test]
async fn unsupported_server_is_rejected_before_login() {
    let mut service = service();
    service.body = service.body.replace("HostAgent", "VirtualCenter");
    let server = Server::start(vec![service]);
    let report = discover(server.policy(), &credentials(), InventoryLimits::default())
        .await
        .unwrap();
    assert_eq!(report.primary_error, Some(Error::UnsupportedServer));
    assert_eq!(server.requests().len(), 1);
}

#[tokio::test]
async fn whole_discovery_deadline_has_an_independent_cleanup_budget() {
    let mut replies = minimal();
    // The first two replies return promptly; only the inventory request is slow.
    replies[2].delay = Duration::from_millis(250);
    let server = Server::start(vec![
        replies.remove(0),
        replies.remove(0),
        replies.remove(0),
        logout(),
    ]);
    let timeouts = Timeouts {
        request: Duration::from_secs(2),
        inactivity: Duration::from_secs(2),
        discovery: Duration::from_millis(150),
        cleanup: Duration::from_secs(2),
        ..Timeouts::default()
    };
    let report = discover(
        server.policy().with_timeouts(timeouts).unwrap(),
        &credentials(),
        InventoryLimits::default(),
    )
    .await
    .unwrap();
    assert_eq!(report.primary_error, Some(Error::Deadline));
    assert_eq!(report.cleanup, Cleanup::LoggedOut);
    assert!(report.elapsed_ms < 1500.);
}
#[tokio::test]
async fn a_successful_inventory_with_failed_logout_is_not_success() {
    let mut replies = minimal();
    *replies.last_mut().unwrap() = Reply::fault("RuntimeFault");
    let server = Server::start(replies);
    let report = discover(server.policy(), &credentials(), InventoryLimits::default())
        .await
        .unwrap();
    assert!(report.inventory.is_some());
    assert_eq!(report.primary_error, None);
    assert_eq!(report.cleanup, Cleanup::Unconfirmed(Error::SoapFault));
    assert!(!report.is_success());
}
#[tokio::test]
async fn duplicate_wrong_object_and_missing_traversal_fail() {
    for (reply, expected) in [
        (
            properties(
                "Folder",
                "root",
                &(property("childEntity", "") + &property("childEntity", "")),
            ),
            Error::Schema,
        ),
        (
            properties("Folder", "wrong-object", &property("childEntity", "")),
            Error::Schema,
        ),
        (properties("Folder", "root", ""), Error::MissingProperty),
    ] {
        let server = Server::start(vec![service(), login(), license(), reply, logout()]);
        let report = discover(server.policy(), &credentials(), InventoryLimits::default())
            .await
            .unwrap();
        assert_eq!(report.primary_error, Some(expected));
        assert_eq!(report.cleanup, Cleanup::LoggedOut);
    }
}
#[tokio::test]
async fn actual_body_limit_does_not_trust_content_length() {
    let mut reply = service();
    reply.body = " ".repeat(2 * 1024 * 1024 + 1);
    reply.declared = None;
    reply.chunked = true;
    let server = Server::start(vec![reply]);
    let report = discover(server.policy(), &credentials(), InventoryLimits::default())
        .await
        .unwrap();
    assert_eq!(report.primary_error, Some(Error::ResponseLimit));
}

#[cfg(target_os = "linux")]
mod export_tests {
    use super::*;
    use rvvdk_vsphere::{ExportOptions, LeaseCleanup, export_vm};
    use std::path::PathBuf;
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    struct Output(PathBuf);
    impl Output {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "rvddk-export-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn dest(&self) -> PathBuf {
            self.0.join("artifact")
        }
    }
    impl Drop for Output {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn options(out: &Output) -> ExportOptions {
        let mut options = ExportOptions::probe(30 * 1024 * 1024 * 1024);
        options.probe_only = false;
        options.output = out.dest();
        options.max_encoded_bytes = 1024 * 1024;
        options
    }
    fn selected(power: &str) -> Vec<Reply> {
        let mut replies = full();
        replies.pop();
        replies[6].body = replies[6].body.replace("poweredOn", power);
        replies.push(replies[6].clone());
        replies
    }
    fn granted() -> Reply {
        Reply::soap(
            "ExportVm",
            "<returnval type='HttpNfcLease'>lease-private</returnval>",
        )
    }

    #[tokio::test]
    async fn inspection_reads_tasks_without_acquiring_or_cancelling_them() {
        let mut replies = selected("poweredOn");
        replies.push(properties(
            "VirtualMachine",
            "vm-private",
            &property(
                "recentTask",
                "<ManagedObjectReference type='Task'>PRIVATE-task</ManagedObjectReference>",
            ),
        ));
        replies.push(properties("Task", "PRIVATE-task", &property("info", "<task type='Task'>PRIVATE-task</task><entity type='VirtualMachine'>vm-private</entity><descriptionId>VirtualMachine.exportVm</descriptionId><state>running</state><cancelable>true</cancelable><cancelled>false</cancelled>")));
        replies.push(logout());
        let server = Server::start(replies);
        let mut options = ExportOptions::probe(30 * 1024 * 1024 * 1024);
        options.inspect_only = true;
        let report = export_vm(server.policy(), &credentials(), options)
            .await
            .unwrap();
        assert_eq!(report.primary_error, None);
        assert_eq!(report.lease_cleanup, LeaseCleanup::NotAcquired);
        assert_eq!(report.session_cleanup, Cleanup::LoggedOut);
        assert_eq!(report.recent_tasks[0].state, "running");
        assert_eq!(report.recent_tasks[0].operation, "export_vm");
        assert!(!serde_json::to_string(&report).unwrap().contains("PRIVATE"));
        assert!(!server.requests().iter().any(|r| r.contains("<ExportVm ")
            || r.contains("CancelTask")
            || r.contains("<ShutdownGuest ")));
    }

    #[tokio::test]
    async fn inspection_rejects_task_identity_change_and_always_logs_out() {
        let mut replies = selected("poweredOn");
        replies.push(properties(
            "VirtualMachine",
            "vm-private",
            &property(
                "recentTask",
                "<ManagedObjectReference type='Task'>PRIVATE-task</ManagedObjectReference>",
            ),
        ));
        replies.push(properties(
            "Task",
            "PRIVATE-task",
            &property("info", "<task type='Task'>wrong</task>"),
        ));
        replies.push(logout());
        let server = Server::start(replies);
        let mut options = ExportOptions::probe(30 * 1024 * 1024 * 1024);
        options.inspect_only = true;
        let report = export_vm(server.policy(), &credentials(), options)
            .await
            .unwrap();
        assert_eq!(report.primary_error, Some(Error::Identity));
        assert_eq!(report.session_cleanup, Cleanup::LoggedOut);
        assert_eq!(report.lease_cleanup, LeaseCleanup::NotAcquired);
    }
    #[tokio::test]
    async fn inspection_bounds_and_validates_all_references_before_task_queries() {
        let task = "<ManagedObjectReference type='Task'>PRIVATE-task</ManagedObjectReference>";
        for (refs, error) in [
            (task.repeat(33), Error::InventoryLimit),
            (task.repeat(2), Error::Schema),
            (
                "<ManagedObjectReference type='VirtualMachine'>PRIVATE-vm</ManagedObjectReference>"
                    .to_owned(),
                Error::Schema,
            ),
        ] {
            let mut replies = selected("poweredOn");
            replies.extend([
                properties(
                    "VirtualMachine",
                    "vm-private",
                    &property("recentTask", &refs),
                ),
                logout(),
            ]);
            let server = Server::start(replies);
            let mut options = ExportOptions::probe(30 * 1024 * 1024 * 1024);
            options.inspect_only = true;
            let report = export_vm(server.policy(), &credentials(), options)
                .await
                .unwrap();
            assert_eq!(report.primary_error, Some(error));
            assert_eq!(report.session_cleanup, Cleanup::LoggedOut);
            assert_eq!(report.lease_cleanup, LeaseCleanup::NotAcquired);
            assert!(
                !server
                    .requests()
                    .iter()
                    .any(|r| r.contains("<type>Task</type>"))
            );
        }
    }

    #[tokio::test]
    async fn inspection_does_not_serialize_arbitrary_task_strings() {
        for (description, timestamp, expected) in [
            ("PRIVATE-description", "2026-01-02T03:04:05.123456Z", None),
            (
                "VirtualMachine.exportVm",
                "PRIVATE-timestamp",
                Some(Error::Schema),
            ),
        ] {
            let mut replies = selected("poweredOn");
            replies.push(properties(
                "VirtualMachine",
                "vm-private",
                &property(
                    "recentTask",
                    "<ManagedObjectReference type='Task'>PRIVATE-task</ManagedObjectReference>",
                ),
            ));
            replies.push(properties("Task", "PRIVATE-task", &property("info", &format!("<task type='Task'>PRIVATE-task</task><descriptionId>{description}</descriptionId><state>error</state><cancelable>true</cancelable><cancelled>true</cancelled><queueTime>{timestamp}</queueTime>"))));
            replies.push(logout());
            let server = Server::start(replies);
            let mut options = ExportOptions::probe(30 * 1024 * 1024 * 1024);
            options.inspect_only = true;
            let report = export_vm(server.policy(), &credentials(), options)
                .await
                .unwrap();
            assert_eq!(report.primary_error, expected);
            assert_eq!(report.session_cleanup, Cleanup::LoggedOut);
            assert!(!serde_json::to_string(&report).unwrap().contains("PRIVATE"));
            if expected.is_none() {
                assert_eq!(report.recent_tasks[0].operation, "other");
                assert_eq!(report.recent_tasks[0].queued_at.as_deref(), Some(timestamp));
            }
        }
    }

    fn ready(url: &str, pin: &str) -> Reply {
        properties(
            "HttpNfcLease",
            "lease-private",
            &(property("state", "ready")
                + &property(
                    "info",
                    &format!(
                        "<lease type='HttpNfcLease'>lease-private</lease><entity type='VirtualMachine'>vm-private</entity><deviceUrl><key>private-disk</key><url>{url}</url><sslThumbprint>{pin}</sslThumbprint><disk>true</disk></deviceUrl><totalDiskCapacityInKB>31457280</totalDiskCapacityInKB><leaseTimeout>30</leaseTimeout>"
                    ),
                )),
        )
    }
    fn payload() -> String {
        "KDMV".to_owned() + &"x".repeat(64 * 1024 - 4)
    }
    fn data() -> Reply {
        let mut reply = Reply::soap("unused", "");
        reply.body = payload();
        reply
    }
    fn manifest() -> Reply {
        let digest = Sha256::digest(payload().as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        Reply::soap(
            "HttpNfcLeaseGetManifest",
            &format!(
                "<returnval><key>private-disk</key><disk>true</disk><size>65536</size><capacity>32212254720</capacity><checksumType>sha256</checksumType><checksum>{digest}</checksum><sha1/></returnval>"
            ),
        )
    }
    fn complete() -> Reply {
        Reply::soap("HttpNfcLeaseComplete", "")
    }
    fn abort() -> Reply {
        Reply::soap("HttpNfcLeaseAbort", "")
    }
    #[tokio::test]
    async fn probe_distinguishes_license_and_power_without_shutdown() {
        for (fault, expected) in [
            ("RestrictedVersionFault", Error::LicenseRestricted),
            ("InvalidPowerStateFault", Error::InvalidPowerState),
            ("MethodDisabledFault", Error::MethodDisabled),
            ("InvalidStateFault", Error::InvalidState),
            ("TaskInProgressFault", Error::TaskInProgress),
        ] {
            let mut replies = selected("poweredOff");
            replies.extend([Reply::fault(fault), logout()]);
            let server = Server::start(replies);
            let report = export_vm(
                server.policy(),
                &credentials(),
                ExportOptions::probe(30 * 1024 * 1024 * 1024),
            )
            .await
            .unwrap();
            assert_eq!(report.primary_error, Some(expected));
            assert_eq!(
                report.lease_cleanup,
                if expected == Error::TaskInProgress {
                    LeaseCleanup::Unconfirmed
                } else {
                    LeaseCleanup::Rejected
                }
            );
            assert_eq!(report.session_cleanup, Cleanup::LoggedOut);
            assert!(!report.shutdown_requested);
            assert!(
                !server
                    .requests()
                    .iter()
                    .any(|r| r.contains("<ShutdownGuest "))
            );
        }
    }
    #[tokio::test]
    async fn powered_on_probe_never_acquires_a_lease_or_changes_power() {
        let mut replies = selected("poweredOn");
        replies.push(logout());
        let server = Server::start(replies);
        let report = export_vm(
            server.policy(),
            &credentials(),
            ExportOptions::probe(30 * 1024 * 1024 * 1024),
        )
        .await
        .unwrap();
        assert_eq!(report.primary_error, Some(Error::InvalidPowerState));
        assert_eq!(report.lease_cleanup, LeaseCleanup::NotAcquired);
        assert_eq!(report.session_cleanup, Cleanup::LoggedOut);
        assert!(!report.shutdown_requested);
        assert!(
            !server
                .requests()
                .iter()
                .any(|r| r.contains("<ExportVm ") || r.contains("<ShutdownGuest "))
        );
    }

    #[tokio::test]
    async fn probe_aborts_any_granted_lease() {
        let mut replies = selected("poweredOff");
        replies.extend([granted(), abort(), logout()]);
        let server = Server::start(replies);
        let report = export_vm(
            server.policy(),
            &credentials(),
            ExportOptions::probe(30 * 1024 * 1024 * 1024),
        )
        .await
        .unwrap();
        assert_eq!(report.primary_error, None);
        assert_eq!(report.lease_cleanup, LeaseCleanup::Aborted);
        assert_eq!(report.session_cleanup, Cleanup::LoggedOut);
        assert!(!report.is_success());
    }
    #[tokio::test]
    async fn opaque_lease_reference_is_escaped_and_aborted() {
        let encoded = "lease:session[synthetic]&amp;&lt;&quot;&apos;";
        let mut grant = granted();
        grant.body = grant.body.replace("lease-private", encoded);
        let mut replies = selected("poweredOff");
        replies.extend([grant, abort(), logout()]);
        let server = Server::start(replies);
        let report = export_vm(
            server.policy(),
            &credentials(),
            ExportOptions::probe(30 * 1024 * 1024 * 1024),
        )
        .await
        .unwrap();
        assert_eq!(report.primary_error, None);
        assert_eq!(report.lease_cleanup, LeaseCleanup::Aborted);
        assert_eq!(report.session_cleanup, Cleanup::LoggedOut);
        assert!(
            server
                .requests()
                .iter()
                .any(|r| r.contains("<HttpNfcLeaseAbort ") && r.contains(encoded))
        );
        assert!(
            !serde_json::to_string(&report)
                .unwrap()
                .contains("synthetic")
        );
    }

    #[tokio::test]
    async fn auxiliary_non_disk_entries_are_not_downloaded_or_verified_as_disks() {
        let auxiliary = "<deviceUrl><key>auxiliary-private</key><disk>false</disk><url>https://other.invalid/never-fetch</url></deviceUrl>";
        let mut info = ready("ENDPOINT/nfc/PRIVATE-ticket", "THUMBPRINT");
        info.body = info.body.replace(
            "<totalDiskCapacityInKB>",
            &format!("{auxiliary}<totalDiskCapacityInKB>"),
        );
        let mut digest = manifest();
        digest.body = digest.body.replace("</HttpNfcLeaseGetManifestResponse>", "<returnval><key>auxiliary-private</key><disk>false</disk></returnval></HttpNfcLeaseGetManifestResponse>");
        let mut replies = selected("poweredOff");
        replies.extend([granted(), info, data(), digest, complete(), logout()]);
        let server = Server::start(replies);
        let out = Output::new();
        let report = export_vm(server.policy(), &credentials(), options(&out))
            .await
            .unwrap();
        assert!(report.is_success(), "{report:?}");
        assert_eq!(report.ignored_non_disk_devices, 1);
        assert_eq!(report.files.len(), 1);
        assert_eq!(
            server
                .requests()
                .iter()
                .filter(|r| r.starts_with("GET "))
                .count(),
            1
        );
        assert!(!server.requests().iter().any(|r| r.contains("never-fetch")));
    }

    #[tokio::test]
    async fn full_transfer_verifies_manifest_completes_logs_out_and_publishes() {
        let mut replies = selected("poweredOff");
        replies.extend([
            granted(),
            ready("ENDPOINT/nfc/PRIVATE-ticket", "THUMBPRINT"),
            data(),
            manifest(),
            complete(),
            logout(),
        ]);
        let server = Server::start(replies);
        let out = Output::new();
        let report = export_vm(server.policy(), &credentials(), options(&out))
            .await
            .unwrap();
        assert!(report.is_success(), "{report:?}");
        assert_eq!(report.received_encoded_bytes, 65536);
        assert_eq!(
            std::fs::read(out.dest().join("disk-1.vmdk")).unwrap(),
            payload().as_bytes()
        );
        assert!(!serde_json::to_string(&report).unwrap().contains("PRIVATE"));
        let requests = server.requests();
        let get = requests
            .iter()
            .find(|r| r.starts_with("GET "))
            .unwrap()
            .to_ascii_lowercase();
        assert!(
            !get.contains("cookie:")
                && !get.contains("authorization:")
                && !get.contains("password")
        );
        assert!(requests.last().unwrap().contains("<Logout "));
    }
    #[tokio::test]
    async fn endpoint_or_pin_rejection_aborts_without_get() {
        for (url, pin) in [
            ("https://other.invalid/nfc/private", "THUMBPRINT"),
            ("ENDPOINT/sdk", "THUMBPRINT"),
            (
                "ENDPOINT/nfc/private",
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
        ] {
            let mut replies = selected("poweredOff");
            replies.extend([granted(), ready(url, pin), abort(), logout()]);
            let server = Server::start(replies);
            let out = Output::new();
            let report = export_vm(server.policy(), &credentials(), options(&out))
                .await
                .unwrap();
            assert_eq!(report.primary_error, Some(Error::DataEndpoint));
            assert_eq!(report.lease_cleanup, LeaseCleanup::Aborted);
            assert_eq!(report.session_cleanup, Cleanup::LoggedOut);
            assert!(!out.dest().exists());
            assert!(!server.requests().iter().any(|r| r.starts_with("GET ")));
            assert_eq!(std::fs::read_dir(&out.0).unwrap().count(), 0);
        }
    }
    #[tokio::test]
    async fn digest_mismatch_retains_both_cleanup_outcomes_without_publication() {
        let mut wrong = manifest();
        wrong.body = wrong.body.replace("65536", "65535");
        let mut replies = selected("poweredOff");
        replies.extend([
            granted(),
            ready("ENDPOINT/nfc/private", "THUMBPRINT"),
            data(),
            wrong,
            Reply::fault("RuntimeFault"),
            Reply::fault("RuntimeFault"),
        ]);
        let server = Server::start(replies);
        let out = Output::new();
        let report = export_vm(server.policy(), &credentials(), options(&out))
            .await
            .unwrap();
        assert_eq!(report.primary_error, Some(Error::Manifest));
        assert_eq!(report.received_encoded_bytes, 65536);
        assert_eq!(report.lease_cleanup, LeaseCleanup::Unconfirmed);
        assert_eq!(report.lease_cleanup_error, Some(Error::SoapFault));
        assert_eq!(
            report.session_cleanup,
            Cleanup::Unconfirmed(Error::SoapFault)
        );
        assert!(!out.dest().exists());
        assert_eq!(std::fs::read_dir(&out.0).unwrap().count(), 0);
    }
    #[tokio::test]
    async fn cancel_pending_download_aborts_and_removes_private_files() {
        let mut slow = data();
        slow.delay = Duration::from_millis(500);
        let mut replies = selected("poweredOff");
        replies.extend([
            granted(),
            ready("ENDPOINT/nfc/private", "THUMBPRINT"),
            slow,
            abort(),
            logout(),
        ]);
        let server = Server::start(replies);
        let out = Output::new();
        let options = options(&out);
        let token = options.cancellation.clone();
        let worker = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(150)).await;
            token.cancel();
        });
        let report = export_vm(server.policy(), &credentials(), options)
            .await
            .unwrap();
        worker.await.unwrap();
        assert_eq!(report.primary_error, Some(Error::Cancelled));
        assert_eq!(report.lease_cleanup, LeaseCleanup::Aborted);
        assert_eq!(report.session_cleanup, Cleanup::LoggedOut);
        assert_eq!(std::fs::read_dir(&out.0).unwrap().count(), 0);
    }
    #[tokio::test]
    async fn output_collision_precedes_power_and_lease_calls() {
        let mut replies = selected("poweredOn");
        replies.push(logout());
        let server = Server::start(replies);
        let out = Output::new();
        std::fs::create_dir(out.dest()).unwrap();
        std::fs::write(out.dest().join("sentinel"), b"keep").unwrap();
        let mut options = options(&out);
        options.allow_graceful_shutdown = true;
        let report = export_vm(server.policy(), &credentials(), options)
            .await
            .unwrap();
        assert_eq!(report.primary_error, Some(Error::Artifact));
        assert!(!report.shutdown_requested);
        assert_eq!(report.lease_cleanup, LeaseCleanup::NotAcquired);
        assert_eq!(std::fs::read(out.dest().join("sentinel")).unwrap(), b"keep");
    }
    #[tokio::test]
    async fn malformed_acquisition_is_uncertain_and_not_retried() {
        let mut replies = selected("poweredOff");
        replies.extend([
            Reply::soap(
                "ExportVm",
                "<returnval type='VirtualMachine'>wrong</returnval>",
            ),
            logout(),
        ]);
        let server = Server::start(replies);
        let report = export_vm(
            server.policy(),
            &credentials(),
            ExportOptions::probe(30 * 1024 * 1024 * 1024),
        )
        .await
        .unwrap();
        assert_eq!(report.primary_error, Some(Error::LeaseUnconfirmed));
        assert_eq!(report.lease_cleanup, LeaseCleanup::Unconfirmed);
        assert_eq!(
            server
                .requests()
                .iter()
                .filter(|r| r.contains("<ExportVm "))
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn graceful_shutdown_revalidates_disk_before_acquisition() {
        for changed in [false, true] {
            let mut replies = selected("poweredOn");
            let mut off = selected("poweredOff").pop().unwrap();
            if changed {
                off.body = off.body.replace("PRIVATE-path", "PRIVATE-replaced");
            }
            replies.extend([Reply::soap("ShutdownGuest", ""), off]);
            if !changed {
                replies.push(Reply::fault("RestrictedVersionFault"));
            }
            replies.push(logout());
            let server = Server::start(replies);
            let out = Output::new();
            let mut options = options(&out);
            options.allow_graceful_shutdown = true;
            let report = export_vm(server.policy(), &credentials(), options)
                .await
                .unwrap();
            assert!(report.shutdown_requested);
            assert_eq!(
                report.last_observed_power_state.as_deref(),
                Some("poweredOff")
            );
            assert_eq!(
                report.primary_error,
                Some(if changed {
                    Error::Identity
                } else {
                    Error::LicenseRestricted
                })
            );
            assert_eq!(report.session_cleanup, Cleanup::LoggedOut);
            assert_eq!(
                server
                    .requests()
                    .iter()
                    .filter(|r| r.contains("<ExportVm "))
                    .count(),
                usize::from(!changed)
            );
            assert_eq!(std::fs::read_dir(&out.0).unwrap().count(), 0);
        }
    }

    #[tokio::test]
    async fn shutdown_failure_never_falls_back_to_hard_power_off() {
        let mut replies = selected("poweredOn");
        replies.extend([Reply::fault("ToolsUnavailableFault"), logout()]);
        let server = Server::start(replies);
        let out = Output::new();
        let mut options = options(&out);
        options.allow_graceful_shutdown = true;
        let report = export_vm(server.policy(), &credentials(), options)
            .await
            .unwrap();
        assert_eq!(report.primary_error, Some(Error::SoapFault));
        assert_eq!(report.lease_cleanup, LeaseCleanup::NotAcquired);
        assert!(server.requests().last().unwrap().contains("<Logout "));
        assert!(
            !server
                .requests()
                .iter()
                .any(|r| r.contains("PowerOffVM") || r.contains("<ExportVm "))
        );
    }

    #[tokio::test]
    async fn transfer_limit_redirect_and_truncation_abort_without_publication() {
        for case in 0..3 {
            let mut bad = data();
            let expected = match case {
                0 => {
                    bad.declared = Some(2 * 1024 * 1024);
                    Error::TransferLimit
                }
                1 => {
                    bad.status = 302;
                    bad.headers = "Location: https://other.invalid/PRIVATE\r\n".to_owned();
                    Error::Http
                }
                _ => {
                    bad.declared = Some(65537);
                    Error::Transport
                }
            };
            let mut replies = selected("poweredOff");
            replies.extend([
                granted(),
                ready("ENDPOINT/nfc/private", "THUMBPRINT"),
                bad,
                abort(),
                logout(),
            ]);
            let server = Server::start(replies);
            let out = Output::new();
            let report = export_vm(server.policy(), &credentials(), options(&out))
                .await
                .unwrap();
            assert_eq!(report.primary_error, Some(expected), "{report:?}");
            assert_eq!(report.lease_cleanup, LeaseCleanup::Aborted);
            assert_eq!(report.session_cleanup, Cleanup::LoggedOut);
            assert_eq!(std::fs::read_dir(&out.0).unwrap().count(), 0);
        }
    }

    #[tokio::test]
    async fn complete_or_logout_failure_prevents_publication() {
        for failed_complete in [false, true] {
            let mut replies = selected("poweredOff");
            replies.extend([
                granted(),
                ready("ENDPOINT/nfc/private", "THUMBPRINT"),
                data(),
                manifest(),
            ]);
            if failed_complete {
                replies.extend([Reply::fault("RuntimeFault"), abort(), logout()]);
            } else {
                replies.extend([complete(), Reply::fault("RuntimeFault")]);
            }
            let server = Server::start(replies);
            let out = Output::new();
            let report = export_vm(server.policy(), &credentials(), options(&out))
                .await
                .unwrap();
            assert!(!report.is_success());
            assert_eq!(
                report.lease_cleanup,
                if failed_complete {
                    LeaseCleanup::Aborted
                } else {
                    LeaseCleanup::Completed
                }
            );
            assert_eq!(std::fs::read_dir(&out.0).unwrap().count(), 0);
        }
    }

    #[tokio::test]
    async fn heartbeats_renew_lease_while_data_response_is_pending() {
        let mut slow = data();
        slow.delay = Duration::from_millis(2300);
        let mut short_lease = ready("ENDPOINT/nfc/private", "THUMBPRINT");
        short_lease.body = short_lease
            .body
            .replace("<leaseTimeout>30", "<leaseTimeout>3");
        let mut replies = selected("poweredOff");
        replies.extend([
            granted(),
            short_lease,
            slow,
            manifest(),
            complete(),
            logout(),
        ]);
        let server = Server::start(replies);
        let out = Output::new();
        let report = export_vm(server.policy(), &credentials(), options(&out))
            .await
            .unwrap();
        assert!(report.is_success(), "{report:?}");
        let requests = server.requests();
        let get = requests.iter().position(|r| r.starts_with("GET ")).unwrap();
        let manifest = requests
            .iter()
            .position(|r| r.contains("<HttpNfcLeaseGetManifest "))
            .unwrap();
        assert!(
            requests[get + 1..manifest]
                .iter()
                .filter(|r| r.contains("<HttpNfcLeaseProgress "))
                .count()
                >= 2
        );
    }

    #[tokio::test]
    async fn wildcard_and_legacy_sha1_are_bound_to_the_sha256_pinned_endpoint() {
        let digest = sha1::Sha1::digest(payload().as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let manifest = Reply::soap(
            "HttpNfcLeaseGetManifest",
            &format!(
                "<returnval><key>private-disk</key><disk>true</disk><size>65536</size><sha1>{digest}</sha1></returnval>"
            ),
        );
        let mut replies = selected("poweredOff");
        replies.extend([
            granted(),
            ready("WILDCARD/ha-nfc/private", "LEGACY_PIN"),
            data(),
            manifest,
            complete(),
            logout(),
        ]);
        let server = Server::start(replies);
        let out = Output::new();
        let report = export_vm(server.policy(), &credentials(), options(&out))
            .await
            .unwrap();
        assert!(report.is_success(), "{report:?}");
    }
}
