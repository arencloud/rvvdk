use crate::{BackingError, Descriptor, Limits};
use std::io::{self, Read};

/// Owned bounded bytes; parse results borrow this object, not a self-reference.
pub struct DescriptorText {
    bytes: Vec<u8>,
    limits: Limits,
    text_len: usize,
}
impl DescriptorText {
    /// Read at most descriptor_bytes + one probe byte, without trusting size hints.
    /// The bound includes terminal NUL padding. Only a contiguous NUL suffix is
    /// excluded from text parsing; original bytes remain available via as_bytes.
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
        // Find the text boundary once, after EOF and total-byte admission. The
        // unpadded case inspects only the last byte; parse() never rescans padding.
        let text_len = bytes
            .iter()
            .rposition(|&byte| byte != 0)
            .map_or(0, |i| i + 1);
        let text = Self {
            bytes,
            limits,
            text_len,
        };
        text.parse()?;
        Ok(text)
    }
    /// Original acquired bytes, including any terminal NUL padding.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Text prefix passed to the strict descriptor parser.
    pub fn text_bytes(&self) -> &[u8] {
        &self.bytes[..self.text_len]
    }
    /// Number of terminal NUL bytes, included in the acquisition byte limit.
    pub fn padding_bytes(&self) -> usize {
        self.bytes.len() - self.text_len
    }
    pub fn parse(&self) -> Result<Descriptor<'_>, crate::DescriptorError> {
        Descriptor::parse_with_limits(self.text_bytes(), self.limits)
    }
}
