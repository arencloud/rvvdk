#[path = "support/sparse_metadata.rs"]
mod f;
use rvvdk_core::{BlockDevice, Capabilities, CopyEndpoint, DiskGeometry, EndpointIdentity};
use rvvdk_vmdk::{
    Descriptor, Limits, SparseDescriptor, SparseMetadata, SparseMetadataError as E,
    SparseMetadataLimits,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
fn text() -> String {
    f::descriptor("monolithicSparse", "RW 2048 SPARSE \"disk.vmdk\"")
}
fn load(bytes: &[u8], text: &str) -> Result<SparseMetadata, E> {
    SparseMetadata::load(
        &SparseDescriptor::parse(text.as_bytes()).unwrap(),
        0,
        &f::Resolver(f::device(bytes)),
        SparseMetadataLimits::default(),
    )
}
#[test]
fn explicit_sparse_subset_preserves_default_parser_rejection() {
    let t = text();
    let d = SparseDescriptor::parse(t.as_bytes()).unwrap();
    assert_eq!(d.size_bytes(), 1048576);
    assert_eq!(d.cid(), 0x12345678);
    assert_eq!(d.metadata().len(), 0);
    for value in ["0", "1", "35cf2bd", "00000001", "ABCDEF01", "ffffffff"] {
        let short = t.replace("12345678", value);
        assert_eq!(
            SparseDescriptor::parse(short.as_bytes()).unwrap().cid(),
            u32::from_str_radix(value, 16).unwrap()
        );
    }
    for value in ["", "123456789", "-1234567", "zzzzzzzz"] {
        assert!(SparseDescriptor::parse(t.replace("12345678", value).as_bytes()).is_err());
    }
    assert!(Descriptor::parse(t.as_bytes()).is_err());
    let mut padded = t.as_bytes().to_vec();
    padded.extend([0; 77]);
    assert!(SparseDescriptor::parse(&padded).is_ok());
    assert!(
        SparseDescriptor::parse_with_limits(
            &padded,
            Limits {
                descriptor_bytes: padded.len() - 1,
                ..Limits::default()
            }
        )
        .is_err()
    );
    for t in [
        t.replace("SPARSE", "FLAT"),
        t.replace("ffffffff", "12345678"),
        t.replace("disk.vmdk\"", "disk.vmdk\" 0"),
        t.replace("2048", "0"),
        t.replace("monolithicSparse", "streamOptimized"),
        format!("{t}RW 2048 SPARSE \"disk.vmdk\"\n"),
        format!("{t}\0hidden"),
    ] {
        assert!(SparseDescriptor::parse(t.as_bytes()).is_err(), "{t}");
    }
    assert!(
        SparseDescriptor::parse(
            f::descriptor("twoGbMaxExtentSparse", "RW 4194305 SPARSE \"disk.vmdk\"").as_bytes()
        )
        .is_err()
    );
}
#[test]
fn retained_map_and_exact_budget_accounting() {
    let t = text();
    let b = f::bytes(Some(&t));
    let m = load(&b, &t).unwrap();
    assert_eq!(&m.grain_sectors()[..3], &[128, 256, 0]);
    assert_eq!(m.grain_sectors().len(), 16);
    assert_eq!(m.cid(), 0x12345678);
    assert_eq!(m.extent_index(), 0);
    assert_eq!(m.logical_offset(), 0);
    m.revalidate().unwrap();
    assert_eq!(m.metadata_read_bytes(), 16384);
    assert_eq!(m.reserved_memory_bytes(), 11488);
    let limits = SparseMetadataLimits {
        memory_bytes: m.reserved_memory_bytes(),
        read_bytes: m.metadata_read_bytes(),
        ..SparseMetadataLimits::default()
    };
    let d = SparseDescriptor::parse(t.as_bytes()).unwrap();
    assert!(SparseMetadata::load(&d, 0, &f::Resolver(f::device(&b)), limits).is_ok());
    for l in [
        SparseMetadataLimits {
            memory_bytes: limits.memory_bytes - 1,
            ..limits
        },
        SparseMetadataLimits {
            read_bytes: limits.read_bytes - 1,
            ..limits
        },
    ] {
        assert!(matches!(
            SparseMetadata::load(&d, 0, &f::Resolver(f::device(&b)), l),
            Err(E::Limit(_))
        ));
    }
}
#[test]
fn external_split_binding_empty_region_and_extent_index() {
    let t = f::descriptor(
        "twoGbMaxExtentSparse",
        "RW 2048 SPARSE \"first.vmdk\"\nRDONLY 2048 SPARSE \"disk.vmdk\"",
    );
    let d = SparseDescriptor::parse(t.as_bytes()).unwrap();
    let r = f::Resolver(f::device(&f::bytes(None)));
    let m = SparseMetadata::load(&d, 1, &r, SparseMetadataLimits::default()).unwrap();
    assert_eq!(m.logical_offset(), 1048576);
    assert_eq!(m.extent_index(), 1);
    assert!(matches!(
        SparseMetadata::load(&d, 2, &r, SparseMetadataLimits::default()),
        Err(E::Invalid("extent index"))
    ));
}
#[test]
fn descriptor_binding_rejects_missing_conflicting_and_capacity_mismatch() {
    let t = text();
    for embedded in [None, Some("garbage"), Some("version=1\n")] {
        assert!(load(&f::bytes(embedded), &t).is_err());
    }
    for alternate in [
        t.replace("12345678", "12345679"),
        t.replace("disk.vmdk", "other.vmdk"),
        t.replace("RW ", "RDONLY "),
        t.replace("monolithicSparse", "twoGbMaxExtentSparse"),
    ] {
        assert!(matches!(
            load(&f::bytes(Some(&alternate)), &t),
            Err(E::Invalid("embedded/external descriptor mismatch"))
        ));
    }
    let b = f::bytes(Some(&t));
    assert!(matches!(
        load(&b, &t.replace("2048", "4096")),
        Err(E::Invalid("descriptor/header capacity mismatch"))
    ));
    let split = f::descriptor("twoGbMaxExtentSparse", "RW 2048 SPARSE \"disk.vmdk\"");
    assert!(load(&b, &split).is_err());
}
#[test]
fn metadata_placement_rejects_before_any_grain_table_read() {
    let t = text();
    for (position, value) in [
        (26 * 512, 0),
        (26 * 512, 1),
        (26 * 512, 21),
        (26 * 512, 22),
        (26 * 512, 26),
        (26 * 512, 127),
        (26 * 512, u32::MAX),
        (21 * 512, 27),
        (21 * 512 + 4, 1),
    ] {
        let mut b = f::bytes(Some(&t));
        f::put32(&mut b, position, value);
        let tracked = Arc::new(Tracked::new(b));
        let r = f::Resolver(tracked.clone());
        assert!(
            SparseMetadata::load(
                &SparseDescriptor::parse(t.as_bytes()).unwrap(),
                0,
                &r,
                SparseMetadataLimits::default()
            )
            .is_err()
        );
        assert_eq!(
            tracked.tables.load(Ordering::Relaxed),
            0,
            "{position}/{value}"
        );
    }
}
#[test]
fn redundant_disagreement_unused_slots_and_data_ranges_reject() {
    let t = text();
    let mut b = f::bytes(Some(&t));
    f::put32(&mut b, 22 * 512, 256);
    assert!(matches!(
        load(&b, &t),
        Err(E::Invalid("redundant grain tables disagree"))
    ));
    for (entry, value) in [
        (0, 1),
        (0, 127),
        (0, 129),
        (0, 384),
        (0, u32::MAX),
        (1, 128),
        (16, 128),
        (511, 128),
    ] {
        let mut b = f::bytes(Some(&t));
        for table in [22, 27] {
            f::put32(&mut b, table * 512 + entry * 4, value);
        }
        assert!(load(&b, &t).is_err(), "{entry}/{value}");
    }
}
#[test]
fn absent_redundancy_is_supported_without_recovery_fallback() {
    let t = text();
    let mut b = f::bytes(Some(&t));
    f::put32(&mut b, 8, 1);
    b[48..56].fill(0);
    b[21 * 512..26 * 512].fill(0xa5);
    assert!(load(&b, &t).is_ok());
}
struct Tracked {
    bytes: Vec<u8>,
    calls: AtomicUsize,
    tables: AtomicUsize,
    endpoints: AtomicUsize,
    mode: usize,
}
impl Tracked {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            calls: AtomicUsize::new(0),
            tables: AtomicUsize::new(0),
            endpoints: AtomicUsize::new(0),
            mode: 0,
        }
    }
}
impl BlockDevice for Tracked {
    fn geometry(&self) -> DiskGeometry {
        DiskGeometry::new(self.bytes.len() as u64, 512, 512).unwrap()
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities::READ
    }
    fn copy_endpoint(&self) -> rvvdk_core::Result<CopyEndpoint> {
        let n = self.endpoints.fetch_add(1, Ordering::Relaxed);
        Ok(CopyEndpoint {
            size: self.bytes.len() as u64 - if self.mode == 2 && n > 0 { 1 } else { 0 },
            capabilities: if self.mode == 3 && n > 0 {
                Capabilities::empty()
            } else {
                Capabilities::READ
            },
            identity: Some(EndpointIdentity::LocalFile {
                device: 1,
                inode: if self.mode == 4 && n > 0 { 2 } else { 1 },
            }),
        })
    }
    fn read_at(&self, offset: u64, out: &mut [u8]) -> rvvdk_core::Result<usize> {
        let n = self.calls.fetch_add(1, Ordering::Relaxed);
        if out.len() == 2048 {
            self.tables.fetch_add(1, Ordering::Relaxed);
        }
        if self.mode == 5 && offset >= 10752 {
            return Ok(0);
        }
        if self.mode == 6 && offset >= 10752 {
            return Err(rvvdk_core::Error::Io(
                std::io::ErrorKind::PermissionDenied.into(),
            ));
        }
        let len = out.len().min(if self.mode == 7 { 3 } else { out.len() });
        out[..len].copy_from_slice(&self.bytes[offset as usize..offset as usize + len]);
        if self.mode == 1 && offset == 0 && n > 0 {
            out[0] ^= 1;
        }
        Ok(len)
    }
    fn write_at(&self, _: u64, _: &[u8]) -> rvvdk_core::Result<usize> {
        panic!("no writes")
    }
    fn flush(&self) -> rvvdk_core::Result<()> {
        panic!("no flush")
    }
}
#[test]
fn live_endpoint_header_changes_and_io_failures_propagate() {
    let t = text();
    let d = SparseDescriptor::parse(t.as_bytes()).unwrap();
    for mode in 1..=7 {
        let mut source = Tracked::new(f::bytes(Some(&t)));
        source.mode = mode;
        let result = SparseMetadata::load(
            &d,
            0,
            &f::Resolver(Arc::new(source)),
            SparseMetadataLimits::default(),
        );
        if mode == 7 {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err(), "{mode}");
        }
    }
}
#[test]
fn budget_failures_precede_offset_reads_and_large_allocations() {
    let t = text();
    let source = Arc::new(Tracked::new(f::bytes(Some(&t))));
    for l in [
        SparseMetadataLimits {
            memory_bytes: 0,
            ..SparseMetadataLimits::default()
        },
        SparseMetadataLimits {
            read_bytes: 512,
            ..SparseMetadataLimits::default()
        },
        SparseMetadataLimits {
            descriptor: Limits {
                descriptor_bytes: 1,
                ..Limits::default()
            },
            ..SparseMetadataLimits::default()
        },
    ] {
        let before = source.calls.load(Ordering::Relaxed);
        assert!(
            SparseMetadata::load(
                &SparseDescriptor::parse(t.as_bytes()).unwrap(),
                0,
                &f::Resolver(source.clone()),
                l
            )
            .is_err()
        );
        assert_eq!(source.calls.load(Ordering::Relaxed) - before, 1);
    }
}
