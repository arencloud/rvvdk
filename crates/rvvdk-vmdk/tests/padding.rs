use rvvdk_vmdk::{BackingError, Descriptor, DescriptorText, ErrorKind, Limits};
use std::io::{self, Read};
const TEXT: &[u8] =
    b"version=1\nCID=12345678\nparentCID=ffffffff\ncreateType=\"custom\"\nRW 1 ZERO\n";
fn acquire(bytes: &[u8], limit: usize) -> Result<DescriptorText, BackingError> {
    DescriptorText::read_from(
        bytes,
        Limits {
            descriptor_bytes: limit,
            ..Limits::default()
        },
    )
}
#[test]
fn original_bytes_and_strict_text_view_are_distinct() {
    for padding in [0, 1, 149, 157, 4096, 8192] {
        let mut bytes = TEXT.to_vec();
        bytes.resize(bytes.len() + padding, 0);
        let owned = acquire(&bytes, bytes.len()).unwrap();
        assert_eq!(owned.as_bytes(), bytes);
        assert_eq!(owned.text_bytes(), TEXT);
        assert_eq!(owned.padding_bytes(), padding);
        assert_eq!(owned.parse().unwrap(), Descriptor::parse(TEXT).unwrap());
        // Repeated parsing borrows the cached prefix, without modifying provenance.
        assert_eq!(owned.parse().unwrap().size_bytes(), 512);
        assert_eq!(owned.as_bytes(), bytes);
        if padding > 0 {
            assert_eq!(
                Descriptor::parse(&bytes).unwrap_err().kind,
                ErrorKind::Encoding
            );
        }
    }
}
#[test]
fn suffix_policy_does_not_require_a_final_newline_or_sector_alignment() {
    for text in [TEXT, TEXT.strip_suffix(b"\n").unwrap()] {
        let mut bytes = text.to_vec();
        bytes.extend([0; 7]);
        let owned = acquire(&bytes, bytes.len()).unwrap();
        assert_eq!(owned.text_bytes(), text);
        assert_eq!(owned.padding_bytes(), 7);
    }
}
#[test]
fn embedded_nuls_and_nonzero_suffixes_cannot_hide_text() {
    for tail in [
        b"\0 ".as_slice(),
        b"\0\n",
        b"\0# comment\n",
        b"\0RW 1 ZERO\n",
        b"\0\xff",
        b"\0\0x\0",
    ] {
        let mut bytes = TEXT.to_vec();
        bytes.extend(tail);
        assert!(
            matches!(acquire(&bytes,bytes.len()),Err(BackingError::Descriptor(e)) if e.kind == ErrorKind::Encoding)
        );
    }
    let mut bytes = TEXT.to_vec();
    bytes.insert(10, 0);
    bytes.extend([0; 100]);
    assert!(
        matches!(acquire(&bytes,bytes.len()),Err(BackingError::Descriptor(e)) if e.kind == ErrorKind::Encoding)
    );
}
#[test]
fn empty_all_nul_and_invalid_prefixes_still_fail_validation() {
    for bytes in [
        b"".as_slice(),
        b"\0",
        b"\0\0\0",
        b"invalid\0\0",
        b"version=1\n\0\0",
    ] {
        assert!(matches!(
            acquire(bytes, bytes.len()),
            Err(BackingError::Descriptor(_))
        ));
    }
    let mut bytes = TEXT.to_vec();
    bytes.extend([0; 100]);
    assert!(
        matches!(DescriptorText::read_from(bytes.as_slice(), Limits { line_bytes: 4, ..Limits::default() }), Err(BackingError::Descriptor(e)) if e.kind == ErrorKind::Limit("line bytes"))
    );
}
struct Chunked {
    bytes: Vec<u8>,
    offset: usize,
    chunk: usize,
    interrupt: bool,
    fail_at_end: bool,
}
impl Read for Chunked {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.interrupt {
            self.interrupt = false;
            return Err(io::ErrorKind::Interrupted.into());
        }
        if self.offset == self.bytes.len() && self.fail_at_end {
            return Err(io::ErrorKind::Other.into());
        }
        let n = out
            .len()
            .min(self.chunk)
            .min(self.bytes.len() - self.offset);
        out[..n].copy_from_slice(&self.bytes[self.offset..self.offset + n]);
        self.offset += n;
        Ok(n)
    }
}
#[test]
fn limit_includes_padding_and_oversize_consumes_only_one_probe() {
    let limit = Limits::default().descriptor_bytes;
    let mut bytes = TEXT.to_vec();
    bytes.resize(limit, 0);
    assert_eq!(
        acquire(&bytes, limit).unwrap().padding_bytes(),
        limit - TEXT.len()
    );
    bytes.extend([0; 64]);
    let mut reader = Chunked {
        bytes,
        offset: 0,
        chunk: 4096,
        interrupt: true,
        fail_at_end: false,
    };
    assert!(matches!(
        DescriptorText::read_from(&mut reader, Limits::default()),
        Err(BackingError::Limit("descriptor acquisition bytes"))
    ));
    assert_eq!(reader.offset, limit + 1);
    reader.offset = 0;
    assert!(matches!(
        DescriptorText::read_from(
            &mut reader,
            Limits {
                descriptor_bytes: 0,
                ..Limits::default()
            }
        ),
        Err(BackingError::Limit(_))
    ));
    assert_eq!(reader.offset, 1);
}
#[test]
fn chunk_boundaries_and_interruptions_preserve_padding_and_require_eof() {
    for chunk in [1, 3, TEXT.len() - 1, TEXT.len(), 4096] {
        let mut bytes = TEXT.to_vec();
        bytes.resize(8192, 0);
        let mut reader = Chunked {
            bytes: bytes.clone(),
            offset: 0,
            chunk,
            interrupt: true,
            fail_at_end: false,
        };
        let owned = DescriptorText::read_from(&mut reader, Limits::default()).unwrap();
        assert_eq!(owned.as_bytes(), bytes);
        assert_eq!(owned.text_bytes(), TEXT);
        reader.offset = 0;
        reader.fail_at_end = true;
        assert!(matches!(
            DescriptorText::read_from(&mut reader, Limits::default()),
            Err(BackingError::Io { .. })
        ));
    }
}
