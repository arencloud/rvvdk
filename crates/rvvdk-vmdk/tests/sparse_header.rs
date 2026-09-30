#[path = "support/sparse.rs"]
mod fixture;
use fixture::{header, put32, put64};
use rvvdk_vmdk::{SparseError, SparseHeader, SparseLimits};
use std::io::{self, Cursor, Read};
const FILE: u64 = 65536;
#[test]
fn exact_little_endian_geometry_and_regions() {
    let h = SparseHeader::parse(&header(), FILE).unwrap();
    assert_eq!(h.flags(), 3);
    assert_eq!(h.capacity_bytes(), 1048576);
    assert_eq!(h.grain_bytes(), 65536);
    assert_eq!(h.grain_count(), 16);
    assert_eq!(h.directory_entries(), 1);
    assert_eq!(h.grain_table_bytes(), 2048);
    assert_eq!(h.descriptor().unwrap().offset(), 512);
    assert_eq!(h.descriptor().unwrap().length(), 10240);
    assert_eq!(h.redundant_directory().unwrap().offset(), 10752);
    assert_eq!(h.primary_directory().offset(), 13312);
    assert_eq!(h.primary_directory().end(), 13824);
    assert_eq!(h.overhead_bytes(), FILE);
    assert_eq!(h.minimum_metadata_bytes(), 15872);
}
#[test]
fn external_descriptor_nonredundant_and_optional_newline_flag() {
    let mut b = header();
    put32(&mut b, 8, 0);
    put64(&mut b, 28, 0);
    put64(&mut b, 36, 0);
    put64(&mut b, 48, 0);
    b[73..77].fill(0);
    let h = SparseHeader::parse(&b, FILE).unwrap();
    assert_eq!(h.descriptor(), None);
    assert_eq!(h.redundant_directory(), None);
    assert_eq!(h.minimum_metadata_bytes(), 3072);
}
#[test]
fn truncation_excess_and_magic_fail_without_casts() {
    for n in 0..512 {
        assert!(matches!(
            SparseHeader::parse(&header()[..n], FILE),
            Err(SparseError::HeaderLength)
        ));
    }
    let mut b = header().to_vec();
    b.push(0);
    assert!(matches!(
        SparseHeader::parse(&b, FILE),
        Err(SparseError::HeaderLength)
    ));
    let mut b = header();
    b[..4].reverse();
    assert!(matches!(
        SparseHeader::parse(&b, FILE),
        Err(SparseError::Magic)
    ));
    let mut unaligned = vec![0];
    unaligned.extend(header());
    assert!(SparseHeader::parse(&unaligned[1..], FILE).is_ok());
}
#[test]
fn rejects_versions_flags_compression_dirty_and_reserved_bytes() {
    for (offset, value) in [
        (4, 0),
        (4, 2),
        (4, 3),
        (8, 4),
        (8, 1 << 16),
        (8, 1 << 17),
        (8, 1 << 31),
        (44, 0),
        (44, 1024),
    ] {
        let mut b = header();
        put32(&mut b, offset, value);
        assert!(matches!(
            SparseHeader::parse(&b, FILE),
            Err(SparseError::Unsupported(_))
        ));
    }
    for offset in [72, 77, 78, 79, 511] {
        let mut b = header();
        b[offset] = 1;
        assert!(matches!(
            SparseHeader::parse(&b, FILE),
            Err(SparseError::Unsupported(_))
        ));
    }
    for offset in 73..77 {
        let mut b = header();
        b[offset] = 0;
        assert!(matches!(
            SparseHeader::parse(&b, FILE),
            Err(SparseError::Invalid("newline check"))
        ));
    }
}
#[test]
fn grain_capacity_and_directory_rounding() {
    for grain in [0, 1, 8, 9, 127, 129] {
        let mut b = header();
        put64(&mut b, 20, grain);
        assert!(SparseHeader::parse(&b, FILE).is_err());
    }
    for capacity in [0, 1, 127, 2049] {
        let mut b = header();
        put64(&mut b, 12, capacity);
        assert!(SparseHeader::parse(&b, FILE).is_err());
    }
    for (grains, entries, bytes) in [
        (512, 1, 512),
        (513, 2, 512),
        (65536, 128, 512),
        (65537, 129, 1024),
    ] {
        let mut b = header();
        put64(&mut b, 12, grains * 128);
        put64(&mut b, 64, 2048);
        let h = SparseHeader::parse(&b, 1048576).unwrap();
        assert_eq!(h.directory_entries(), entries);
        assert_eq!(h.primary_directory().length(), bytes);
    }
}
#[test]
fn metadata_ranges_pairs_overlap_and_file_bounds() {
    for (offset, value) in [
        (28, 0),
        (36, 0),
        (48, 0),
        (56, 0),
        (56, u64::MAX),
        (64, 0),
        (64, 127),
        (28, 128),
        (48, 128),
        (56, 128),
        (56, 21),
        (56, 1),
        (48, 1),
    ] {
        let mut b = header();
        put64(&mut b, offset, value);
        assert!(SparseHeader::parse(&b, FILE).is_err(), "{offset}/{value}");
    }
    assert!(matches!(
        SparseHeader::parse(&header(), FILE - 1),
        Err(SparseError::Bounds("overhead"))
    ));
    let mut b = header();
    put32(&mut b, 8, 1);
    assert!(matches!(
        SparseHeader::parse(&b, FILE),
        Err(SparseError::Invalid(_))
    ));
    let mut b = header();
    put64(&mut b, 12, 32768 * 128);
    assert!(matches!(
        SparseHeader::parse(&b, FILE),
        Err(SparseError::Bounds("minimum metadata storage"))
    ));
}
#[test]
fn checked_arithmetic_and_independent_limits() {
    let unlimited = SparseLimits {
        capacity_bytes: u64::MAX,
        grain_bytes: u64::MAX,
        descriptor_bytes: u64::MAX,
        directory_entries: u64::MAX,
        metadata_bytes: u64::MAX,
    };
    for (offset, value) in [
        (12, u64::MAX - 127),
        (20, 1 << 63),
        (28, u64::MAX),
        (36, u64::MAX),
        (48, u64::MAX),
        (56, u64::MAX - 1),
        (64, u64::MAX - 127),
    ] {
        let mut b = header();
        put64(&mut b, offset, value);
        if offset == 20 {
            put64(&mut b, 12, 1 << 63);
        }
        assert!(
            matches!(
                SparseHeader::parse_with_limits(&b, u64::MAX, unlimited),
                Err(SparseError::Overflow(_))
            ),
            "{offset}"
        );
    }
    let exact = SparseLimits {
        capacity_bytes: 1048576,
        grain_bytes: 65536,
        descriptor_bytes: 10240,
        directory_entries: 1,
        metadata_bytes: 65536,
    };
    assert!(SparseHeader::parse_with_limits(&header(), FILE, exact).is_ok());
    for l in [
        SparseLimits {
            capacity_bytes: 1048575,
            ..exact
        },
        SparseLimits {
            grain_bytes: 65535,
            ..exact
        },
        SparseLimits {
            descriptor_bytes: 10239,
            ..exact
        },
        SparseLimits {
            directory_entries: 0,
            ..exact
        },
        SparseLimits {
            metadata_bytes: 65535,
            ..exact
        },
    ] {
        assert!(matches!(
            SparseHeader::parse_with_limits(&header(), FILE, l),
            Err(SparseError::Limit(_))
        ));
    }
}
struct Short {
    bytes: Cursor<Vec<u8>>,
    interrupt: bool,
    fail: bool,
}
impl Read for Short {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.interrupt {
            self.interrupt = false;
            return Err(io::ErrorKind::Interrupted.into());
        }
        if self.fail {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        let n = out.len().min(3);
        self.bytes.read(&mut out[..n])
    }
}
#[test]
fn acquisition_reads_only_one_sector_and_propagates_short_io() {
    let mut bytes = header().to_vec();
    bytes.extend([0xa5; 100]);
    let mut reader = Short {
        bytes: Cursor::new(bytes),
        interrupt: true,
        fail: false,
    };
    SparseHeader::read_from(&mut reader, FILE, SparseLimits::default()).unwrap();
    assert_eq!(reader.bytes.position(), 512);
    reader.bytes = Cursor::new(header()[..511].to_vec());
    assert!(
        matches!(SparseHeader::read_from(&mut reader,FILE,SparseLimits::default()),Err(SparseError::Io(e)) if e.kind()==io::ErrorKind::UnexpectedEof)
    );
    reader.fail = true;
    assert!(
        matches!(SparseHeader::read_from(reader,FILE,SparseLimits::default()),Err(SparseError::Io(e)) if e.kind()==io::ErrorKind::PermissionDenied)
    );
}
#[test]
fn deterministic_mutations_never_panic_or_escape_admitted_bounds() {
    for index in 0..512 {
        for value in [0, 1, 0x7f, 0xff] {
            let mut b = header();
            b[index] = value;
            if let Ok(h) = SparseHeader::parse(&b, FILE) {
                assert!(
                    h.capacity_bytes() > 0 && h.capacity_bytes().is_multiple_of(h.grain_bytes())
                );
                assert!(h.minimum_metadata_bytes() <= h.overhead_bytes());
                for r in [
                    Some(h.primary_directory()),
                    h.redundant_directory(),
                    h.descriptor(),
                ]
                .into_iter()
                .flatten()
                {
                    assert!(r.offset() >= 512 && r.length() > 0 && r.end() <= FILE);
                }
            }
        }
    }
}
