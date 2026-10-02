use crate::{Error, Result, xml};
use roxmltree::Node;
use serde::Serialize;
use std::{collections::BTreeMap, fmt};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum Kind {
    Folder,
    Datacenter,
    ComputeResource,
    ClusterComputeResource,
    ResourcePool,
    HostSystem,
    Datastore,
    VirtualMachine,
    SessionManager,
    PropertyCollector,
    LicenseManager,
    HttpNfcLease,
    Task,
}
impl Kind {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Folder => "Folder",
            Self::Datacenter => "Datacenter",
            Self::ComputeResource => "ComputeResource",
            Self::ClusterComputeResource => "ClusterComputeResource",
            Self::ResourcePool => "ResourcePool",
            Self::HostSystem => "HostSystem",
            Self::Datastore => "Datastore",
            Self::VirtualMachine => "VirtualMachine",
            Self::SessionManager => "SessionManager",
            Self::PropertyCollector => "PropertyCollector",
            Self::LicenseManager => "LicenseManager",
            Self::HttpNfcLease => "HttpNfcLease",
            Self::Task => "Task",
        }
    }
    fn parse(s: &str) -> Result<Self> {
        [
            Self::Folder,
            Self::Datacenter,
            Self::ComputeResource,
            Self::ClusterComputeResource,
            Self::ResourcePool,
            Self::HostSystem,
            Self::Datastore,
            Self::VirtualMachine,
            Self::SessionManager,
            Self::PropertyCollector,
            Self::LicenseManager,
            Self::HttpNfcLease,
            Self::Task,
        ]
        .into_iter()
        .find(|k| k.name() == s)
        .ok_or(Error::Schema)
    }
    pub(crate) fn paths(self) -> &'static [&'static str] {
        match self {
            Self::Folder => &["childEntity"],
            Self::Datacenter => &["vmFolder", "hostFolder", "datastore"],
            Self::ComputeResource | Self::ClusterComputeResource => &["host", "resourcePool"],
            Self::ResourcePool => &["vm", "resourcePool"],
            Self::HostSystem => &["summary.hardware", "runtime.connectionState"],
            Self::Datastore => &["summary", "info"],
            Self::VirtualMachine => &[
                "runtime.powerState",
                "config.guestId",
                "config.uuid",
                "config.hardware.device",
                "config.template",
                "guest.toolsRunningStatus",
                "snapshot",
                "disabledMethod",
            ],
            Self::LicenseManager => &["licenses", "licensedEdition"],
            Self::HttpNfcLease => &["state", "info", "error"],
            Self::Task => &["info"],
            _ => &[],
        }
    }
}
/// Managed references are intentionally absent from serialized discovery evidence.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Reference {
    pub(crate) kind: Kind,
    value: String,
    escaped_value: String,
}
impl fmt::Debug for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Reference")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}
impl Reference {
    pub(crate) fn parse(node: Node<'_, '_>) -> Result<Self> {
        let kind = Kind::parse(xml::reference_type(node).ok_or(Error::Schema)?)?;
        let value = xml::text(node)?;
        if value.is_empty() || value.len() > 256 {
            return Err(Error::Schema);
        }
        Ok(Self {
            kind,
            value: value.to_owned(),
            escaped_value: xml::escape(value)
                .map_err(|_| Error::Schema)?
                .replace('\r', "&#13;"),
        })
    }
    pub(crate) fn element(&self, name: &str) -> String {
        format!(
            "<{name} type='{}'>{}</{name}>",
            self.kind.name(),
            self.escaped_value
        )
    }
}
#[derive(Clone, Copy, Debug)]
pub struct InventoryLimits {
    pub max_objects: usize,
    pub max_disks_per_vm: usize,
}
impl Default for InventoryLimits {
    fn default() -> Self {
        Self {
            max_objects: 128,
            max_disks_per_vm: 16,
        }
    }
}
impl InventoryLimits {
    pub(crate) fn validate(self) -> Result<()> {
        if self.max_objects == 0
            || self.max_objects > 128
            || self.max_disks_per_vm == 0
            || self.max_disks_per_vm > 16
        {
            Err(Error::InvalidInput)
        } else {
            Ok(())
        }
    }
}
#[derive(Debug, Serialize)]
pub struct About {
    pub version: String,
    pub build: u64,
    pub api_version: String,
    pub api_type: &'static str,
}
#[derive(Debug, Serialize)]
pub struct Inventory {
    pub about: About,
    pub hosts: Vec<Host>,
    pub datastores: Vec<Datastore>,
    pub vms: Vec<Vm>,
    pub license: License,
}
#[derive(Debug, Serialize)]
pub struct Host {
    pub connection: String,
    pub cpu_threads: u64,
    pub memory_bytes: u64,
}
#[derive(Debug, Serialize)]
pub struct Datastore {
    pub label: String,
    pub kind: String,
    pub capacity_bytes: u64,
    pub free_bytes: u64,
    pub accessible: bool,
    pub vmfs_version: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct License {
    pub available_editions: Vec<String>,
    pub active_assignment: &'static str,
}
/// Private identity is available explicitly to callers, never via Debug or serde.
/// Revalidate it on the same admitted host before future mutating operations.
pub struct VmIdentity {
    pub(crate) reference: Reference,
    bios_uuid: String,
}
impl fmt::Debug for VmIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("VmIdentity([redacted])")
    }
}
impl VmIdentity {
    pub fn managed_reference(&self) -> &str {
        &self.reference.value
    }
    pub fn bios_uuid(&self) -> &str {
        &self.bios_uuid
    }
}
#[derive(Debug, Serialize)]
pub struct Vm {
    pub label: String,
    pub guest_id: String,
    pub power_state: String,
    pub tools: String,
    pub template: bool,
    pub snapshot_present: bool,
    pub export_disabled_method_list: bool,
    pub disks: Vec<Disk>,
    #[serde(skip)]
    pub identity: VmIdentity,
}
#[derive(Debug, Serialize)]
pub struct Disk {
    pub capacity_bytes: u64,
    pub backing_type: String,
    pub disk_mode: String,
    pub thin: Option<bool>,
    pub encrypted: bool,
    pub parent_present: bool,
    #[serde(skip)]
    pub(crate) identity: DiskIdentity,
}
impl Disk {
    /// Explicit private identity access for source selection; omitted from serde/Debug.
    pub fn device_key(&self) -> Option<u64> {
        self.identity.key
    }
    /// Backing identity may contain datastore/guest names. Keep it private and
    /// revalidate with the VM reference, UUID and device key on the same endpoint.
    pub fn backing_identity(&self) -> Option<&str> {
        self.identity.backing.as_deref()
    }
}
#[derive(PartialEq, Eq)]
pub(crate) struct DiskIdentity {
    pub(crate) key: Option<u64>,
    pub(crate) backing: Option<String>,
}
impl fmt::Debug for DiskIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DiskIdentity([redacted])")
    }
}

pub(crate) struct Properties<'a, 'i>(BTreeMap<&'a str, Node<'a, 'i>>);
impl<'a, 'i> Properties<'a, 'i> {
    pub(crate) fn parse(response: Node<'a, 'i>, reference: &Reference) -> Result<Self> {
        Self::parse_fields(response, reference, reference.kind.paths())
    }
    pub(crate) fn parse_fields(
        response: Node<'a, 'i>,
        reference: &Reference,
        fields: &[&str],
    ) -> Result<Self> {
        let result = xml::required(response, "returnval")?;
        let object = xml::required(result, "objects")?;
        if Reference::parse(xml::required(object, "obj")?)? != *reference {
            return Err(Error::Schema);
        }
        let mut props = BTreeMap::new();
        for entry in object.children().filter(Node::is_element) {
            if entry.has_tag_name((xml::VIM, "missingSet")) {
                return Err(Error::MissingProperty);
            }
            if !entry.has_tag_name((xml::VIM, "propSet")) {
                continue;
            }
            let name = xml::text(xml::required(entry, "name")?)?;
            if !fields.contains(&name) {
                return Err(Error::Schema);
            }
            if props.insert(name, xml::required(entry, "val")?).is_some() {
                return Err(Error::Schema);
            }
        }
        Ok(Self(props))
    }
    pub(crate) fn get(&self, name: &str) -> Result<Node<'a, 'i>> {
        self.0.get(name).copied().ok_or(Error::MissingProperty)
    }
    pub(crate) fn optional(&self, name: &str) -> Option<Node<'a, 'i>> {
        self.0
            .get(name)
            .copied()
            .filter(|n| !matches!(n.attribute((xml::XSI, "nil")), Some("true" | "1")))
    }
    pub(crate) fn references(&self, name: &str) -> Result<Vec<Reference>> {
        let node = self.get(name)?;
        if matches!(node.attribute((xml::XSI, "nil")), Some("true" | "1")) {
            return Err(Error::MissingProperty);
        }
        if xml::reference_type(node).is_some() {
            return Ok(vec![Reference::parse(node)?]);
        }
        let mut references = Vec::new();
        for child in node.children().filter(Node::is_element) {
            if references.len() == 128 {
                return Err(Error::InventoryLimit);
            }
            references.push(Reference::parse(child)?);
        }
        Ok(references)
    }
    pub(crate) fn host(&self) -> Result<Host> {
        let hardware = self.get("summary.hardware")?;
        Ok(Host {
            connection: xml::scalar(self.get("runtime.connectionState")?)?,
            cpu_threads: xml::number(xml::required(hardware, "numCpuThreads")?)?,
            memory_bytes: xml::number(xml::required(hardware, "memorySize")?)?,
        })
    }
    pub(crate) fn datastore(&self, index: usize) -> Result<Datastore> {
        let summary = self.get("summary")?;
        let vmfs = xml::child(self.get("info")?, "vmfs")?;
        let capacity_bytes = xml::number(xml::required(summary, "capacity")?)?;
        let free_bytes = xml::number(xml::required(summary, "freeSpace")?)?;
        if free_bytes > capacity_bytes {
            return Err(Error::Schema);
        }
        Ok(Datastore {
            label: format!("datastore-{index}"),
            kind: xml::scalar(xml::required(summary, "type")?)?,
            capacity_bytes,
            free_bytes,
            accessible: xml::boolean(xml::required(summary, "accessible")?)?,
            vmfs_version: vmfs
                .map(|v| xml::scalar(xml::required(v, "version")?))
                .transpose()?,
        })
    }
    pub(crate) fn license(&self) -> Result<License> {
        let mut editions = Vec::new();
        for license in self.get("licenses")?.children().filter(Node::is_element) {
            if editions.len() == 128 {
                return Err(Error::InventoryLimit);
            }
            editions.push(xml::scalar(xml::required(license, "editionKey")?)?);
        }
        Ok(License {
            available_editions: editions,
            active_assignment: "unresolved",
        })
    }
    pub(crate) fn vm(&self, reference: Reference, index: usize, limit: usize) -> Result<Vm> {
        let uuid = xml::text(self.get("config.uuid")?)?;
        if uuid.len() != 36 || !uuid.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-') {
            return Err(Error::Schema);
        }
        let mut disks = Vec::new();
        for device in self
            .get("config.hardware.device")?
            .children()
            .filter(Node::is_element)
        {
            if xsi_type(device)? != "VirtualDisk" {
                continue;
            }
            if disks.len() == limit {
                return Err(Error::InventoryLimit);
            }
            let backing = xml::required(device, "backing")?;
            let capacity_kib = xml::number(xml::required(device, "capacityInKB")?)?;
            let capacity = xml::child(device, "capacityInBytes")?
                .map(xml::number)
                .transpose()?
                .unwrap_or(capacity_kib.checked_mul(1024).ok_or(Error::Schema)?);
            if capacity == 0 || capacity / 1024 != capacity_kib || capacity % 512 != 0 {
                return Err(Error::Schema);
            }
            disks.push(Disk {
                capacity_bytes: capacity,
                backing_type: xsi_type(backing)?.to_owned(),
                disk_mode: xml::scalar(xml::required(backing, "diskMode")?)?,
                thin: xml::child(backing, "thinProvisioned")?
                    .map(xml::boolean)
                    .transpose()?,
                encrypted: xml::child(backing, "keyId")?.is_some(),
                parent_present: xml::child(backing, "parent")?.is_some(),
                identity: DiskIdentity {
                    key: xml::child(device, "key")?.map(xml::number).transpose()?,
                    backing: xml::child(backing, "fileName")?
                        .map(|n| xml::text(n).map(str::to_owned))
                        .transpose()?,
                },
            });
        }
        let export_disabled_method_list = self
            .optional("disabledMethod")
            .map(|n| {
                n.children()
                    .filter(Node::is_element)
                    .any(|n| matches!(n.text(), Some("ExportVm" | "exportVm")))
            })
            .unwrap_or(false);
        Ok(Vm {
            label: format!("vm-{index}"),
            guest_id: xml::scalar(self.get("config.guestId")?)?,
            power_state: xml::scalar(self.get("runtime.powerState")?)?,
            tools: xml::scalar(self.get("guest.toolsRunningStatus")?)?,
            template: xml::boolean(self.get("config.template")?)?,
            snapshot_present: self.optional("snapshot").is_some(),
            export_disabled_method_list,
            disks,
            identity: VmIdentity {
                reference,
                bios_uuid: uuid.to_owned(),
            },
        })
    }
}
fn xsi_type<'a>(node: Node<'a, '_>) -> Result<&'a str> {
    let value = node.attribute((xml::XSI, "type")).ok_or(Error::Schema)?;
    let (prefix, local) = value
        .split_once(':')
        .map_or((None, value), |(p, l)| (Some(p), l));
    if node.lookup_namespace_uri(prefix) != Some(xml::VIM)
        || local.len() > 100
        || !local.bytes().all(|b| b.is_ascii_alphanumeric())
    {
        return Err(Error::Schema);
    }
    Ok(local)
}

#[cfg(test)]
mod reference_tests {
    use super::*;
    #[test]
    fn opaque_references_round_trip_without_xml_injection() {
        for value in [
            "lease:session[synthetic]",
            "opaque / value",
            "a<&\"'>",
            "λ-reference",
            "opaque\r\n\tvalue",
        ] {
            let raw = format!(
                "<r xmlns='urn:vim25' type='HttpNfcLease'>{}</r>",
                xml::escape(value).unwrap().replace('\r', "&#13;")
            );
            let doc = xml::parse(&raw).unwrap();
            let reference = Reference::parse(doc.root_element()).unwrap();
            assert_eq!(reference.value, value);
            let encoded = reference.element("_this");
            let decoded = xml::parse(&encoded).unwrap();
            assert_eq!(decoded.root_element().text(), Some(value));
            assert_eq!(decoded.descendants().filter(|n| n.is_element()).count(), 1);
        }
        for value in ["".to_owned(), "x".repeat(257)] {
            let raw = format!("<r type='HttpNfcLease'>{value}</r>");
            assert_eq!(
                Reference::parse(xml::parse(&raw).unwrap().root_element()).unwrap_err(),
                Error::Schema
            );
        }
    }
}
