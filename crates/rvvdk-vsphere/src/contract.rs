//! Bounded source selection and untrusted, versioned export artifact metadata.
//!
//! No network or filesystem operations occur here. Metadata records claims; it
//! never grants lease ownership, cleanup authority, or permission to skip fresh
//! identity/content checks. Operational names and credentials are not persisted.
use crate::{ConnectionPolicy, Disk, Inventory, Vm};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// Fixed single-disk v1 metadata admission budget, before JSON allocation.
pub const MAX_METADATA_BYTES: usize = 4096;
pub const MAX_LOGICAL_BYTES: u64 = 64 << 40;
pub const MAX_CONTAINER_BYTES: u64 = 1 << 40;
pub const CONTAINER_FILE_NAME: &str = "disk-1.vmdk";

/// Closed diagnostics never retain malformed JSON, identifiers or paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ContractError {
    #[error("invalid or over-budget contract input")]
    InvalidInput,
    #[error("unsupported artifact schema version")]
    Version,
    #[error("source identity or endpoint trust does not match")]
    IdentityMismatch,
    #[error("source selection is missing or ambiguous")]
    Selection,
    #[error("source is outside the qualified export scope")]
    UnsupportedScope,
    #[error("artifact is incomplete")]
    Incomplete,
    #[error("observed container does not match artifact metadata")]
    ContentMismatch,
}
type Result<T> = std::result::Result<T, ContractError>;

/// Caller-supplied provenance, not a claim that this library verified the pin
/// independently. Both modes still require the exact leaf-certificate pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PinProvenance {
    ExternallyVerified,
    TrustOnFirstUse,
}

/// Runtime-only endpoint policy. No URL or certificate is serialized or debugged.
#[derive(Clone)]
pub struct EndpointIdentity {
    policy: ConnectionPolicy,
    binding: [u8; 32],
    provenance: PinProvenance,
}
impl EndpointIdentity {
    pub fn pinned(endpoint: &str, pin: &str, provenance: PinProvenance) -> Result<Self> {
        let policy =
            ConnectionPolicy::pinned(endpoint, pin).map_err(|_| ContractError::InvalidInput)?;
        let binding = bind(&[
            b"rvddk.endpoint.v1",
            policy.endpoint.as_str().as_bytes(),
            pin.to_ascii_lowercase().as_bytes(),
            match provenance {
                PinProvenance::ExternallyVerified => b"externally_verified",
                PinProvenance::TrustOnFirstUse => b"trust_on_first_use",
            },
        ]);
        Ok(Self {
            policy,
            binding,
            provenance,
        })
    }
    pub fn connection_policy(&self) -> ConnectionPolicy {
        self.policy.clone()
    }
    pub fn provenance(&self) -> PinProvenance {
        self.provenance
    }
}
impl fmt::Debug for EndpointIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("EndpointIdentity([redacted])")
    }
}

/// Explicit runtime request: names, disk order and capacity alone cannot select.
/// Revalidate against inventory from this endpoint immediately before mutation;
/// this check is an observation, not a snapshot or remote race exclusion.
#[derive(Clone)]
pub struct SourceSelection {
    endpoint: EndpointIdentity,
    vm_reference: String,
    bios_uuid: String,
    disk_key: u64,
    backing: String,
    logical_bytes: u64,
    binding: [u8; 32],
}
impl SourceSelection {
    pub fn new(
        endpoint: EndpointIdentity,
        vm_reference: &str,
        bios_uuid: &str,
        disk_key: u64,
        backing: &str,
        logical_bytes: u64,
    ) -> Result<Self> {
        if !text(vm_reference, 256)
            || !text(backing, 4096)
            || disk_key == 0
            || disk_key > i32::MAX as u64
            || !capacity(logical_bytes)
            || bios_uuid.len() != 36
            || !bios_uuid.bytes().enumerate().all(|(i, b)| {
                if [8, 13, 18, 23].contains(&i) {
                    b == b'-'
                } else {
                    b.is_ascii_hexdigit()
                }
            })
        {
            return Err(ContractError::InvalidInput);
        }
        let bios_uuid = bios_uuid.to_ascii_lowercase();
        let binding = bind(&[
            b"rvddk.source.v1",
            &endpoint.binding,
            vm_reference.as_bytes(),
            bios_uuid.as_bytes(),
            &disk_key.to_le_bytes(),
            backing.as_bytes(),
            &logical_bytes.to_le_bytes(),
        ]);
        Ok(Self {
            endpoint,
            vm_reference: vm_reference.into(),
            bios_uuid,
            disk_key,
            backing: backing.into(),
            logical_bytes,
            binding,
        })
    }
    pub fn logical_bytes(&self) -> u64 {
        self.logical_bytes
    }
    pub fn endpoint(&self) -> &EndpointIdentity {
        &self.endpoint
    }

    /// The caller must obtain `inventory` through `observed_endpoint`; metadata
    /// cannot authenticate that association. Duplicate references fail closed.
    pub fn select<'a>(
        &self,
        observed_endpoint: &EndpointIdentity,
        inventory: &'a Inventory,
    ) -> Result<&'a Disk> {
        if inventory.vms.len() > crate::InventoryLimits::default().max_objects {
            return Err(ContractError::InvalidInput);
        }
        let mut matches = inventory
            .vms
            .iter()
            .filter(|vm| vm.identity.managed_reference() == self.vm_reference);
        let vm = matches.next().ok_or(ContractError::Selection)?;
        if matches.next().is_some() {
            return Err(ContractError::Selection);
        }
        self.check_vm(observed_endpoint, vm)
    }

    pub fn check_vm<'a>(
        &self,
        observed_endpoint: &EndpointIdentity,
        vm: &'a Vm,
    ) -> Result<&'a Disk> {
        if self.endpoint.binding != observed_endpoint.binding
            || vm.identity.managed_reference() != self.vm_reference
            || !vm
                .identity
                .bios_uuid()
                .eq_ignore_ascii_case(&self.bios_uuid)
        {
            return Err(ContractError::IdentityMismatch);
        }
        crate::export::admit_vm(vm, self.logical_bytes)
            .map_err(|_| ContractError::UnsupportedScope)?;
        if vm.power_state != "poweredOff" || vm.export_disabled_method_list {
            return Err(ContractError::UnsupportedScope);
        }
        let disk = &vm.disks[0];
        if disk.identity.key != Some(self.disk_key)
            || disk.identity.backing.as_deref() != Some(&self.backing)
        {
            return Err(ContractError::IdentityMismatch);
        }
        Ok(disk)
    }
}
impl fmt::Debug for SourceSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SourceSelection([redacted])")
    }
}

/// Caller-generated nonzero identifier. Uniqueness and resource ownership must
/// be established by the future job store, never inferred from these bytes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ArtifactId([u8; 16]);
impl ArtifactId {
    pub fn new(bytes: [u8; 16]) -> Result<Self> {
        if bytes == [0; 16] {
            Err(ContractError::InvalidInput)
        } else {
            Ok(Self(bytes))
        }
    }
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}
impl fmt::Debug for ArtifactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ArtifactId([redacted])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Completeness {
    Incomplete,
    Complete,
}

/// Recorded assertions only. A decoded record never proves these checks ran.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationClaim {
    Unchecked,
    ContainerDigestVerified,
    LogicalReadbackVerified,
}

/// Caller observations of the container, not logical capacity. Incomplete
/// records have no digest or validation claim; completed records need a digest.
#[derive(Clone)]
pub struct ArtifactObservation {
    pub container_bytes: u64,
    pub completeness: Completeness,
    pub validation: ValidationClaim,
    pub container_sha256: Option<[u8; 32]>,
}
impl fmt::Debug for ArtifactObservation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ArtifactObservation([redacted])")
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ExportArtifact {
    id: ArtifactId,
    source_binding: [u8; 32],
    provenance: PinProvenance,
    logical_bytes: u64,
    container_bytes: u64,
    completeness: Completeness,
    validation: ValidationClaim,
    digest: Option<[u8; 32]>,
}
impl fmt::Debug for ExportArtifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ExportArtifact([redacted])")
    }
}

// Deliberately private: public Deserialize would bypass the byte budget or
// expose parser errors containing operational input. No paths/URLs/lease refs.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema_version: u32,
    artifact_id: String,
    source_binding: String,
    pin_provenance: PinProvenance,
    container_format: ContainerFormat,
    logical_bytes: u64,
    container_bytes: u64,
    completeness: Completeness,
    validation: ValidationClaim,
    container_sha256: Option<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ContainerFormat {
    StreamOptimizedVmdk,
}

impl ExportArtifact {
    /// Construct internally consistent claims; does not read or validate a file.
    pub fn new(
        id: ArtifactId,
        source: &SourceSelection,
        observed: ArtifactObservation,
    ) -> Result<Self> {
        let value = Self {
            id,
            source_binding: source.binding,
            provenance: source.endpoint.provenance,
            logical_bytes: source.logical_bytes,
            container_bytes: observed.container_bytes,
            completeness: observed.completeness,
            validation: observed.validation,
            digest: observed.container_sha256,
        };
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<()> {
        let consistent = match self.completeness {
            Completeness::Incomplete => {
                self.digest.is_none() && self.validation == ValidationClaim::Unchecked
            }
            Completeness::Complete => self.container_bytes > 0 && self.digest.is_some(),
        };
        if !consistent
            || !capacity(self.logical_bytes)
            || self.container_bytes > MAX_CONTAINER_BYTES
        {
            return Err(ContractError::InvalidInput);
        }
        Ok(())
    }
    pub fn id(&self) -> ArtifactId {
        self.id
    }
    pub fn logical_bytes(&self) -> u64 {
        self.logical_bytes
    }
    pub fn container_bytes(&self) -> u64 {
        self.container_bytes
    }
    pub fn completeness(&self) -> Completeness {
        self.completeness
    }
    pub fn validation_claim(&self) -> ValidationClaim {
        self.validation
    }
    pub fn pin_provenance(&self) -> PinProvenance {
        self.provenance
    }

    /// Compare binding only. Does not authorize remote or local resource actions.
    pub fn check_source(&self, source: &SourceSelection) -> Result<()> {
        if self.source_binding != source.binding
            || self.logical_bytes != source.logical_bytes
            || self.provenance != source.endpoint.provenance
        {
            Err(ContractError::IdentityMismatch)
        } else {
            Ok(())
        }
    }
    /// Compare fresh caller-supplied container observations, regardless of the
    /// stored validation claim. Does not check VMDK structure or logical bytes.
    pub fn check_container(&self, bytes: u64, sha256: &[u8; 32]) -> Result<()> {
        if self.completeness != Completeness::Complete {
            return Err(ContractError::Incomplete);
        }
        if self.container_bytes != bytes || self.digest.as_ref() != Some(sha256) {
            Err(ContractError::ContentMismatch)
        } else {
            Ok(())
        }
    }
    /// Sensitive explicit persistence: opaque hashes still correlate private
    /// sources/content. Keep these bytes private, outside diagnostic reports.
    pub fn to_json(&self) -> Result<Vec<u8>> {
        let wire = Wire {
            schema_version: 1,
            artifact_id: hex(&self.id.0),
            source_binding: hex(&self.source_binding),
            pin_provenance: self.provenance,
            container_format: ContainerFormat::StreamOptimizedVmdk,
            logical_bytes: self.logical_bytes,
            container_bytes: self.container_bytes,
            completeness: self.completeness,
            validation: self.validation,
            container_sha256: self.digest.as_ref().map(|v| hex(v)),
        };
        serde_json::to_vec(&wire).map_err(|_| ContractError::InvalidInput)
    }
    /// Parse at most 4 KiB. Unknown/duplicate fields, versions, formats, malformed
    /// sizes and inconsistent states fail closed. Success returns untrusted claims.
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_METADATA_BYTES {
            return Err(ContractError::InvalidInput);
        }
        let w: Wire = serde_json::from_slice(bytes).map_err(|_| ContractError::InvalidInput)?;
        if w.schema_version != 1 {
            return Err(ContractError::Version);
        }
        let value = Self {
            id: ArtifactId::new(unhex(&w.artifact_id)?)?,
            source_binding: unhex(&w.source_binding)?,
            provenance: w.pin_provenance,
            logical_bytes: w.logical_bytes,
            container_bytes: w.container_bytes,
            completeness: w.completeness,
            validation: w.validation,
            digest: w.container_sha256.as_deref().map(unhex).transpose()?,
        };
        value.validate()?;
        Ok(value)
    }
}

fn text(value: &str, max: usize) -> bool {
    !value.is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}
fn capacity(bytes: u64) -> bool {
    bytes > 0 && bytes <= MAX_LOGICAL_BYTES && bytes.is_multiple_of(512)
}
fn bind(parts: &[&[u8]]) -> [u8; 32] {
    let mut hash = Sha256::new();
    for part in parts {
        hash.update((part.len() as u64).to_le_bytes());
        hash.update(part);
    }
    hash.finalize().into()
}
fn hex(bytes: &[u8]) -> String {
    crate::transport::hex_string(bytes)
}
fn unhex<const N: usize>(value: &str) -> Result<[u8; N]> {
    if value.len() != N * 2
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(ContractError::InvalidInput);
    }
    let mut result = [0; N];
    for (i, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let digit = |b: u8| if b <= b'9' { b - b'0' } else { b - b'a' + 10 };
        result[i] = digit(pair[0]) * 16 + digit(pair[1]);
    }
    Ok(result)
}

#[cfg(test)]
mod tests;
