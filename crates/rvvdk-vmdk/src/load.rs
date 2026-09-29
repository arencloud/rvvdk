use crate::{BackingError, Descriptor, Limits};
use std::io::{self, Read};

/// Owned bounded bytes; parse results borrow this object, not a self-reference.
pub struct DescriptorText {
    bytes: Vec<u8>,
    limits: Limits,
}
impl DescriptorText {
    /// Read at most descriptor_bytes + one probe byte, without trusting size hints.
    /// This bounds bytes/memory, not blocking time. Interrupted reads are retried.
    pub fn read_from(mut reader: impl Read, limits: Limits) -> Result<Self, BackingError> {
        let mut bytes = Vec::new();
        let mut chunk = [0_u8; 4096];
        loop {
            let remaining = limits.descriptor_bytes - bytes.len();
            let count = if remaining == 0 {
                1
            } else {
                remaining.min(chunk.len())
            };
            let n = match reader.read(&mut chunk[..count]) {
                Ok(n) => n,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(BackingError::io("reading descriptor", e)),
            };
            if n == 0 {
                break;
            }
            if remaining == 0 {
                return Err(BackingError::Limit("descriptor acquisition bytes"));
            }
            bytes.extend_from_slice(&chunk[..n]);
        }
        let text = Self { bytes, limits };
        text.parse()?;
        Ok(text)
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn parse(&self) -> Result<Descriptor<'_>, crate::DescriptorError> {
        Descriptor::parse_with_limits(&self.bytes, self.limits)
    }
}
