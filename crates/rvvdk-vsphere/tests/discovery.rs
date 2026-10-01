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
        property("config.hardware.device","<VirtualDevice xsi:type='VirtualDisk'><capacityInKB>31457280</capacityInKB><capacityInBytes>32212254720</capacityInBytes><backing xsi:type='VirtualDiskFlatVer2BackingInfo'><fileName>PRIVATE-path</fileName><diskMode>persistent</diskMode><thinProvisioned>false</thinProvisioned></backing></VirtualDevice>")
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
        listener.set_nonblocking(true).unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let observed = requests.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let halted = stop.clone();
        let worker = thread::spawn(move || {
            let config = Arc::new(config);
            let mut replies = replies.into_iter().peekable();
            while !halted.load(Ordering::Relaxed) && replies.peek().is_some() {
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
                let mut stream =
                    StreamOwned::new(ServerConnection::new(config.clone()).unwrap(), stream);
                while !halted.load(Ordering::Relaxed) && replies.peek().is_some() {
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
                        .unwrap()
                        .parse()
                        .unwrap();
                    assert!(length < 65536);
                    let mut body = vec![0; length];
                    if stream.read_exact(&mut body).is_err() {
                        break;
                    }
                    let body = String::from_utf8(body).unwrap();
                    observed.lock().unwrap().push(format!("{headers}{body}"));
                    let reply = replies.next().unwrap();
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
