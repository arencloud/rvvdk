use super::*;
use rvvdk_vsphere::contract::{ArtifactId, EndpointIdentity, PinProvenance, SourceSelection};
use serde::Deserialize;
use std::{
    fs::OpenOptions,
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Selection {
    vm_reference: String,
    bios_uuid: String,
    disk_key: u64,
    backing: String,
    logical_bytes: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    schema_version: u32,
    endpoint: String,
    certificate_sha256: String,
    pin_provenance: PinProvenance,
    selection: Selection,
    store: PathBuf,
    destination: PathBuf,
    artifact_id: String,
    output_id: String,
    #[serde(default = "timeout")]
    timeout_seconds: u64,
}
fn timeout() -> u64 {
    3600
}
pub(super) struct Request {
    pub store: PathBuf,
    pub destination: PathBuf,
    pub artifact: ArtifactId,
    pub output: OutputId,
    pub source: SourceSelection,
    pub timeout: Duration,
}
fn id(s: &str) -> Result<[u8; 16], Status> {
    if s.len() != 32 || !s.is_ascii() {
        return Err(Status::Invalid);
    }
    let mut bytes = [0; 16];
    for (i, b) in bytes.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(|_| Status::Invalid)?;
    }
    Ok(bytes)
}
fn private(m: &std::fs::Metadata) -> bool {
    // SAFETY: geteuid has no arguments or memory preconditions.
    m.is_file()
        && m.nlink() == 1
        && m.mode() & 0o077 == 0
        && m.uid() == unsafe { libc::geteuid() }
        && m.len() <= 16384
}
fn same(a: &std::fs::Metadata, b: &std::fs::Metadata) -> bool {
    private(b)
        && (a.dev(), a.ino(), a.uid(), a.mode(), a.nlink(), a.len())
            == (b.dev(), b.ino(), b.uid(), b.mode(), b.nlink(), b.len())
}
pub(super) fn read(path: &Path) -> Result<Request, Status> {
    let f = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)
        .map_err(|_| Status::Failed)?;
    let before = f.metadata().map_err(|_| Status::Failed)?;
    if !private(&before) {
        return Err(Status::Failed);
    }
    let mut bytes = Vec::new();
    (&f).take(16385)
        .read_to_end(&mut bytes)
        .map_err(|_| Status::Failed)?;
    let after = f.metadata().map_err(|_| Status::Failed)?;
    if bytes.len() > 16384 || bytes.len() as u64 != before.len() || !same(&before, &after) {
        return Err(Status::Failed);
    }
    parse(&bytes)
}
fn parse(bytes: &[u8]) -> Result<Request, Status> {
    let c: Config = serde_json::from_slice(bytes).map_err(|_| Status::Invalid)?;
    if c.schema_version != 1
        || !(1..=21600).contains(&c.timeout_seconds)
        || [&c.store, &c.destination]
            .iter()
            .any(|p| !p.is_absolute() || p.as_os_str().as_encoded_bytes().contains(&0))
    {
        return Err(Status::Invalid);
    }
    let endpoint = EndpointIdentity::pinned(&c.endpoint, &c.certificate_sha256, c.pin_provenance)
        .map_err(|_| Status::Invalid)?;
    let s = c.selection;
    let source = SourceSelection::new(
        endpoint,
        &s.vm_reference,
        &s.bios_uuid,
        s.disk_key,
        &s.backing,
        s.logical_bytes,
    )
    .map_err(|_| Status::Invalid)?;
    Ok(Request {
        store: c.store,
        destination: c.destination,
        artifact: ArtifactId::new(id(&c.artifact_id)?).map_err(|_| Status::Invalid)?,
        output: OutputId::new(id(&c.output_id)?).map_err(|_| Status::Invalid)?,
        source,
        timeout: Duration::from_secs(c.timeout_seconds),
    })
}
#[cfg(test)]
pub(super) mod tests;
