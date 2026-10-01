//! Independent, bounded standalone ESXi 8.0.3 SOAP discovery.
//!
//! This crate does not export disks, modify VMs or use a VMware SDK. Await
//! [`discover`] to completion: dropping its future (or killing the process) cannot
//! guarantee remote logout. Internally enforced deadlines still attempt cleanup
//! with a separate bounded budget. Debug/errors/reports omit operational secrets.
mod error;
mod inventory;
mod transport;
mod xml;

pub use error::{Error, Result};
pub use inventory::{
    About, Datastore, Disk, Host, Inventory, InventoryLimits, License, Vm, VmIdentity,
};
use inventory::{Kind, Properties, Reference};
use serde::Serialize;
use std::{
    collections::{BTreeSet, VecDeque},
    fmt,
    time::Instant,
};
pub use transport::{ConnectionPolicy, ConnectionReuse, RequestMeasurement, Timeouts};
use transport::{Method, Transport};
use zeroize::Zeroizing;

/// Owned credentials are wiped on drop; HTTP/TLS library copies are not guaranteed
/// to be erased. No password environment variables, files or command arguments are
/// required by the qualification example.
pub struct Credentials {
    user: Zeroizing<String>,
    password: Zeroizing<String>,
}
impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Credentials([redacted])")
    }
}
impl Credentials {
    pub fn new(user: String, password: String) -> Result<Self> {
        let value = Self {
            user: Zeroizing::new(user),
            password: Zeroizing::new(password),
        };
        if value.user.is_empty()
            || value.password.is_empty()
            || value.user.len() > 1024
            || value.password.len() > 1024
        {
            return Err(Error::InvalidInput);
        }
        xml::escape(&value.user)?;
        xml::escape(&value.password)?;
        Ok(value)
    }
}
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Cleanup {
    NotNeeded,
    LoggedOut,
    Unconfirmed(Error),
}
#[derive(Debug, Serialize)]
pub struct DiscoveryReport {
    pub inventory: Option<Inventory>,
    pub primary_error: Option<Error>,
    pub cleanup: Cleanup,
    pub pagination_cleanup_error: Option<Error>,
    pub requests: Vec<RequestMeasurement>,
    /// Successful certificate-pin checks; completed TLS handshakes may be fewer on failures.
    pub certificate_checks: usize,
    pub connection_policy: ConnectionReuse,
    pub elapsed_ms: f64,
}
impl DiscoveryReport {
    pub fn is_success(&self) -> bool {
        self.inventory.is_some()
            && self.primary_error.is_none()
            && self.pagination_cleanup_error.is_none()
            && self.cleanup == Cleanup::LoggedOut
    }
}
struct Session {
    transport: Transport,
    collector: Reference,
    manager: Reference,
    root: Reference,
    license: Reference,
    deadline: Instant,
    cleanup_timeout: std::time::Duration,
    pagination_cleanup_error: Option<Error>,
}

/// Perform one scoped discovery and explicitly report primary and cleanup errors.
/// Only pinned HTTPS /sdk is admitted. No automatic retry, proxy, redirect or
/// mutation methods are exposed. Requires a Tokio runtime with I/O and time enabled.
pub async fn discover(
    policy: ConnectionPolicy,
    credentials: &Credentials,
    limits: InventoryLimits,
) -> Result<DiscoveryReport> {
    limits.validate()?;
    let started = Instant::now();
    let deadline = started + policy.timeouts.discovery;
    let cleanup_timeout = policy.timeouts.cleanup;
    let connection_policy = policy.reuse;
    let mut transport = Transport::new(policy)?;
    let service = transport
        .call(
            Method::RetrieveServiceContent,
            "<_this type='ServiceInstance'>ServiceInstance</_this>".to_owned(),
            deadline,
        )
        .await;
    let setup = service.and_then(|raw| {
        let doc = xml::parse(&raw)?;
        let content = xml::required(xml::response(&doc, "RetrieveServiceContent")?, "returnval")?;
        let about = xml::required(content, "about")?;
        let version = xml::scalar(xml::required(about, "version")?)?;
        let api_version = xml::scalar(xml::required(about, "apiVersion")?)?;
        if xml::text(xml::required(about, "apiType")?)? != "HostAgent"
            || version != "8.0.3"
            || api_version != "8.0.3.0"
        {
            return Err(Error::UnsupportedServer);
        }
        let get = |name, kind| {
            let reference = Reference::parse(xml::required(content, name)?)?;
            if reference.kind != kind {
                Err(Error::Schema)
            } else {
                Ok(reference)
            }
        };
        Ok((
            About {
                version,
                api_version,
                build: xml::number(xml::required(about, "build")?)?,
                api_type: "HostAgent",
            },
            get("sessionManager", Kind::SessionManager)?,
            get("propertyCollector", Kind::PropertyCollector)?,
            get("rootFolder", Kind::Folder)?,
            get("licenseManager", Kind::LicenseManager)?,
        ))
    });
    let (about, manager, collector, root, license) = match setup {
        Ok(setup) => setup,
        Err(error) => {
            return Ok(DiscoveryReport {
                inventory: None,
                primary_error: Some(error),
                cleanup: Cleanup::NotNeeded,
                pagination_cleanup_error: None,
                certificate_checks: transport.handshakes(),
                requests: transport.measurements,
                connection_policy,
                elapsed_ms: started.elapsed().as_secs_f64() * 1000.,
            });
        }
    };
    let mut session = Session {
        transport,
        manager,
        collector,
        root,
        license,
        deadline,
        cleanup_timeout,
        pagination_cleanup_error: None,
    };
    let body = format!(
        "{}<userName>{}</userName><password>{}</password>",
        session.manager.element("_this"),
        xml::escape(&credentials.user)?,
        xml::escape(&credentials.password)?
    );
    // Once Login is attempted, cleanup is always attempted, including ambiguity.
    let login = session
        .transport
        .call(Method::Login, body, deadline)
        .await
        .and_then(|_| {
            if session.transport.has_cookie() {
                Ok(())
            } else {
                Err(Error::Cookie)
            }
        });
    let result = match login {
        Ok(()) => session.inventory(about, limits).await,
        Err(error) => Err(error),
    };
    let logout = session
        .transport
        .call(
            Method::Logout,
            session.manager.element("_this"),
            Instant::now() + cleanup_timeout,
        )
        .await;
    session.transport.clear_cookie();
    let cleanup = match logout {
        Ok(_) => Cleanup::LoggedOut,
        Err(error) => Cleanup::Unconfirmed(error),
    };
    let (inventory, primary_error) = match result {
        Ok(inventory) => (Some(inventory), None),
        Err(error) => (None, Some(error)),
    };
    Ok(DiscoveryReport {
        inventory,
        primary_error,
        cleanup,
        pagination_cleanup_error: session.pagination_cleanup_error,
        certificate_checks: session.transport.handshakes(),
        requests: session.transport.measurements,
        connection_policy,
        elapsed_ms: started.elapsed().as_secs_f64() * 1000.,
    })
}
impl Session {
    async fn properties(&mut self, reference: &Reference) -> Result<Zeroizing<String>> {
        let paths = reference
            .kind
            .paths()
            .iter()
            .map(|p| format!("<pathSet>{p}</pathSet>"))
            .collect::<String>();
        let body = format!(
            "{}<specSet><propSet><type>{}</type><all>false</all>{paths}</propSet><objectSet>{}<skip>false</skip></objectSet></specSet><options><maxObjects>1</maxObjects></options>",
            self.collector.element("_this"),
            reference.kind.name(),
            reference.element("obj")
        );
        let raw = self
            .transport
            .call(Method::RetrievePropertiesEx, body, self.deadline)
            .await?;
        let token = {
            let doc = xml::parse(&raw)?;
            let response = xml::response(&doc, "RetrievePropertiesEx")?;
            let result = xml::required(response, "returnval")?;
            xml::child(result, "token")?
                .map(|n| xml::text(n).map(str::to_owned))
                .transpose()?
        };
        if let Some(token) = token {
            // A one-object retrieval must fit one page. Never silently accept a
            // partial inventory; release any returned server continuation cursor.
            let cleanup = match xml::escape(&token) {
                Ok(token) => self
                    .transport
                    .call(
                        Method::CancelRetrievePropertiesEx,
                        format!("{}<token>{token}</token>", self.collector.element("_this")),
                        Instant::now() + self.cleanup_timeout,
                    )
                    .await
                    .map(|_| ()),
                Err(error) => Err(error),
            };
            self.pagination_cleanup_error = cleanup.err();
            return Err(Error::Pagination);
        }
        Ok(raw)
    }
    async fn inventory(&mut self, about: About, limits: InventoryLimits) -> Result<Inventory> {
        let license_raw = self.properties(&self.license.clone()).await?;
        let doc = xml::parse(&license_raw)?;
        let license =
            Properties::parse(xml::response(&doc, "RetrievePropertiesEx")?, &self.license)?
                .license()?;
        let mut inventory = Inventory {
            about,
            hosts: Vec::new(),
            datastores: Vec::new(),
            vms: Vec::new(),
            license,
        };
        let mut queue = VecDeque::from([self.root.clone()]);
        let mut seen = BTreeSet::new();
        while let Some(reference) = queue.pop_front() {
            if seen.contains(&reference) {
                continue;
            }
            if seen.len() == limits.max_objects {
                return Err(Error::InventoryLimit);
            }
            seen.insert(reference.clone());
            let raw = self.properties(&reference).await?;
            let doc = xml::parse(&raw)?;
            let props =
                Properties::parse(xml::response(&doc, "RetrievePropertiesEx")?, &reference)?;
            let keys: &[&str] = match reference.kind {
                Kind::Folder => &["childEntity"],
                Kind::Datacenter => &["vmFolder", "hostFolder", "datastore"],
                Kind::ComputeResource | Kind::ClusterComputeResource => &["host", "resourcePool"],
                Kind::ResourcePool => &["vm", "resourcePool"],
                Kind::HostSystem => {
                    inventory.hosts.push(props.host()?);
                    &[]
                }
                Kind::Datastore => {
                    inventory
                        .datastores
                        .push(props.datastore(inventory.datastores.len() + 1)?);
                    &[]
                }
                Kind::VirtualMachine => {
                    inventory.vms.push(props.vm(
                        reference.clone(),
                        inventory.vms.len() + 1,
                        limits.max_disks_per_vm,
                    )?);
                    &[]
                }
                _ => return Err(Error::Schema),
            };
            for key in keys {
                for next in props.references(key)? {
                    if queue.len() == limits.max_objects {
                        return Err(Error::InventoryLimit);
                    }
                    queue.push_back(next);
                }
            }
        }
        Ok(inventory)
    }
}
