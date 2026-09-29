mod support;
use rvvdk_core::{
    BlockDevice, Capabilities, CopyEndpoint, DiskGeometry, EndpointIdentity, Error, ExtentKind,
    MemoryBlockDevice, RawDisk, VirtualDisk,
};
use rvvdk_datamover::{CopyOptions, DataMover};
use rvvdk_vmdk::{Descriptor, ResolutionLimits, ResolvedDescriptor, VmdkDisk};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use support::*;

#[test]
fn all_ranges_match_flat_zero_offset_and_repeated_source_oracle() {
    let (r, bytes) = patterned();
    let d = disk(
        &descriptor("RW 2 FLAT \"x\" 3\nRW 1 ZERO\nRDONLY 1 FLAT \"x\" 0\nRW 1 FLAT \"x\" 7"),
        &r,
    );
    let expected = [
        &bytes[1536..2560],
        &[0; 512][..],
        &bytes[..512],
        &bytes[3584..4096],
    ]
    .concat();
    assert_eq!(d.size(), expected.len() as u64);
    for offset in 0..=expected.len() {
        for len in [0, 1, 3, 513, expected.len() - offset] {
            if offset + len > expected.len() {
                continue;
            }
            let mut output = vec![0xcc; len];
            assert_eq!(d.read_at(offset as u64, &mut output).unwrap(), len);
            assert_eq!(output, expected[offset..offset + len]);
        }
    }
    assert_eq!(d.geometry().logical_block_size(), 512);
    assert!(d.is_read_only());
    assert_eq!(d.copy_endpoint().unwrap().identity, None);
}
#[test]
fn range_errors_precede_buffer_changes_and_all_mutations_are_unsupported() {
    let (r, _) = patterned();
    let d = disk(&descriptor("RW 1 FLAT \"x\" 0"), &r);
    for offset in [511, 513, u64::MAX] {
        let mut b = [17; 2];
        assert!(d.read_at(offset, &mut b).is_err());
        assert_eq!(b, [17; 2]);
    }
    assert_eq!(d.read_at(512, &mut []).unwrap(), 0);
    assert!(matches!(
        d.read_at(513, &mut []),
        Err(Error::OutOfBounds { .. })
    ));
    assert!(matches!(
        d.extents(u64::MAX, 1),
        Err(Error::RangeOverflow { .. })
    ));
    assert!(matches!(d.extents(512, 1), Err(Error::OutOfBounds { .. })));
    assert!(d.extents(512, 0).unwrap().is_empty());
    assert!(matches!(d.write_at(0, &[1]), Err(Error::Unsupported)));
    assert!(matches!(d.write_zero_at(0, 512), Err(Error::Unsupported)));
    assert!(matches!(d.discard(0, 512), Err(Error::Unsupported)));
    assert!(matches!(d.flush(), Err(Error::Unsupported)));
    assert!(
        !d.capabilities()
            .intersects(Capabilities::WRITE | Capabilities::DIRECT_IO | Capabilities::FLUSH)
    );
}
#[test]
fn extents_are_clipped_coalesced_and_never_report_physical_holes() {
    let (r, _) = patterned();
    let d = disk(
        &descriptor(
            "RW 1 FLAT \"x\" 0\nRW 1 FLAT \"y\" 1\nRW 1 ZERO\nRW 2 ZERO\nRW 1 FLAT \"x\" 3",
        ),
        &r,
    );
    let e = d.extents(17, 3000).unwrap();
    assert_eq!(
        e.iter()
            .map(|e| (e.offset(), e.length(), e.kind()))
            .collect::<Vec<_>>(),
        vec![
            (17, 1007, ExtentKind::Data),
            (1024, 1536, ExtentKind::Zero),
            (2560, 457, ExtentKind::Data)
        ]
    );
    for offset in [0, 511, 512, 1023, 1024, 2559, 2560, 3071] {
        let single = d.extents(offset, 1).unwrap();
        assert_eq!(single.len(), 1);
        assert_eq!(single[0].offset(), offset);
        assert_eq!(single[0].length(), 1);
    }
}
#[test]
fn maximum_capacity_zero_extent_and_last_byte_do_not_overflow() {
    let (r, _) = patterned();
    let d = disk(&descriptor("RW 36028797018963967 ZERO"), &r);
    let mut bytes = [1; 3];
    d.read_exact_at(d.size() - 3, &mut bytes).unwrap();
    assert_eq!(bytes, [0; 3]);
    let extent = d.extents(d.size() - 3, 3).unwrap();
    assert_eq!(extent[0].end(), d.size());
}
struct ShortSource {
    eof: AtomicBool,
    denied: AtomicBool,
    reads: AtomicUsize,
}
impl BlockDevice for ShortSource {
    fn geometry(&self) -> DiskGeometry {
        DiskGeometry::new(4096, 512, 512).unwrap()
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities::READ
    }
    fn copy_endpoint(&self) -> rvvdk_core::Result<CopyEndpoint> {
        Ok(CopyEndpoint {
            size: if self.eof.load(Ordering::Relaxed) {
                0
            } else {
                4096
            },
            capabilities: self.capabilities(),
            identity: Some(EndpointIdentity::Memory { address: 1 }),
        })
    }
    fn read_at(&self, _: u64, buf: &mut [u8]) -> rvvdk_core::Result<usize> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        if self.denied.load(Ordering::Relaxed) {
            return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied).into());
        }
        if self.eof.load(Ordering::Relaxed) {
            return Ok(0);
        }
        let n = buf.len().min(7);
        buf[..n].fill(23);
        Ok(n)
    }
    fn write_at(&self, _: u64, _: &[u8]) -> rvvdk_core::Result<usize> {
        panic!("no writes")
    }
    fn flush(&self) -> rvvdk_core::Result<()> {
        panic!("no flush")
    }
}
#[test]
fn partial_reads_complete_but_eof_and_io_errors_are_not_zero_filled() {
    let source = Arc::new(ShortSource {
        eof: AtomicBool::new(false),
        denied: AtomicBool::new(false),
        reads: AtomicUsize::new(0),
    });
    let r = Source(source.clone());
    let d = disk(&descriptor("RW 1 ZERO\nRW 1 FLAT \"x\" 0"), &r);
    let mut bytes = [99; 1024];
    d.read_exact_at(0, &mut bytes).unwrap();
    assert_eq!(&bytes[..512], &[0; 512]);
    assert_eq!(&bytes[512..], &[23; 512]);
    assert_eq!(source.reads.load(Ordering::Relaxed), 74);
    source.eof.store(true, Ordering::Relaxed);
    bytes.fill(99);
    assert!(matches!(
        d.read_at(0, &mut bytes),
        Err(Error::UnexpectedEof { .. })
    ));
    assert_eq!(&bytes[..512], &[0; 512]);
    assert_eq!(&bytes[512..], &[99; 512]);
    assert!(d.copy_endpoint().is_err());
    source.denied.store(true, Ordering::Relaxed);
    assert!(matches!(d.read_at(512, &mut [0; 1]), Err(Error::Io(_))));
}
#[test]
fn concurrent_reads_share_immutable_mapping_and_backing_handles() {
    let (r, bytes) = patterned();
    let d = disk(&descriptor("RW 2 FLAT \"x\" 1\nRW 1 ZERO"), &r);
    std::thread::scope(|s| {
        for _ in 0..4 {
            let d = &d;
            let expected = &bytes[512..1536];
            s.spawn(move || {
                for _ in 0..50 {
                    let mut b = [0; 1536];
                    d.read_exact_at(0, &mut b).unwrap();
                    assert_eq!(&b[..1024], expected);
                    assert_eq!(&b[1024..], &[0; 512]);
                }
            });
        }
    });
}
#[test]
fn portable_copy_preserves_data_zero_and_destination_tail() {
    let (r, bytes) = patterned();
    let d = disk(
        &descriptor("RW 2 FLAT \"x\" 1\nRW 1 ZERO\nRW 1 FLAT \"x\" 5"),
        &r,
    );
    for workers in [1, 4] {
        let target = RawDisk::new(MemoryBlockDevice::new(2560).unwrap());
        target.write_all_at(0, &[0xa5; 2560]).unwrap();
        let mover = DataMover::new(CopyOptions::with_concurrency(512, 4096, workers).unwrap());
        let plan = mover.plan_with_destination(&d, &target).unwrap();
        let report = mover.execute_plan(&plan, &d, &target).unwrap();
        let mut actual = [0; 2560];
        target.read_exact_at(0, &mut actual).unwrap();
        assert_eq!(&actual[..1024], &bytes[512..1536]);
        assert_eq!(&actual[1024..1536], &[0; 512]);
        assert_eq!(&actual[1536..2048], &bytes[2560..3072]);
        assert_eq!(&actual[2048..], &[0xa5; 512]);
        let _ = report;
        rvvdk_datamover::Verifier::new(d.size(), 257, 4096)
            .unwrap()
            .verify(&d, &target)
            .unwrap();
    }
}
struct AliasTarget(Arc<dyn BlockDevice>);
impl VirtualDisk for AliasTarget {
    fn geometry(&self) -> DiskGeometry {
        self.0.geometry()
    }
    fn capabilities(&self) -> Capabilities {
        self.0.capabilities()
    }
    fn copy_endpoint(&self) -> rvvdk_core::Result<CopyEndpoint> {
        self.0.copy_endpoint()
    }
    fn read_at(&self, o: u64, b: &mut [u8]) -> rvvdk_core::Result<usize> {
        self.0.read_at(o, b)
    }
    fn write_at(&self, _: u64, _: &[u8]) -> rvvdk_core::Result<usize> {
        panic!("alias must reject before writes")
    }
    fn write_zero_at(&self, _: u64, _: u64) -> rvvdk_core::Result<()> {
        panic!("no zero")
    }
    fn discard(&self, _: u64, _: u64) -> rvvdk_core::Result<()> {
        panic!("no discard")
    }
    fn flush(&self) -> rvvdk_core::Result<()> {
        panic!("no flush")
    }
    fn extents(&self, _: u64, _: u64) -> rvvdk_core::Result<Vec<rvvdk_core::Extent>> {
        Err(Error::Unsupported)
    }
}
#[test]
fn composite_alias_checks_apply_to_direct_planned_and_trait_object_copies() {
    let (r, _) = patterned();
    let d = disk(&descriptor("RW 1 FLAT \"x\" 1\nRW 1 ZERO"), &r);
    let target = AliasTarget(r.0.clone());
    let mover = DataMover::new(CopyOptions::default());
    assert!(matches!(
        mover.copy(&d, &target),
        Err(Error::AliasedEndpoints)
    ));
    assert!(matches!(
        mover.plan_with_destination(&d, &target),
        Err(Error::AliasedEndpoints)
    ));
    let plan = mover.plan(&d).unwrap();
    let dyn_source: &dyn VirtualDisk = &d;
    assert!(matches!(
        mover.execute_plan(&plan, dyn_source, &target),
        Err(Error::AliasedEndpoints)
    ));
    assert!(
        d.validate_destination_identity(CopyEndpoint {
            size: 4096,
            capabilities: Capabilities::WRITE,
            identity: None
        })
        .is_err()
    );
}
#[test]
fn constructor_and_execute_preflight_reject_late_truncation_before_writes() {
    let s = Arc::new(ShortSource {
        eof: AtomicBool::new(false),
        denied: AtomicBool::new(false),
        reads: AtomicUsize::new(0),
    });
    let r = Source(s.clone());
    let t = descriptor("RW 1 FLAT \"x\" 0");
    let resolved = ResolvedDescriptor::resolve(
        &Descriptor::parse(t.as_bytes()).unwrap(),
        &r,
        ResolutionLimits::default(),
    )
    .unwrap();
    s.eof.store(true, Ordering::Relaxed);
    assert!(VmdkDisk::new(resolved).is_err());
    s.eof.store(false, Ordering::Relaxed);
    let d = disk(&t, &r);
    let target = RawDisk::new(MemoryBlockDevice::new(512).unwrap());
    target.write_all_at(0, &[41; 512]).unwrap();
    let mover = DataMover::new(CopyOptions::default());
    let plan = mover.plan(&d).unwrap();
    s.eof.store(true, Ordering::Relaxed);
    assert!(mover.execute_plan(&plan, &d, &target).is_err());
    let mut b = [0; 512];
    target.read_exact_at(0, &mut b).unwrap();
    assert_eq!(b, [41; 512]);
}

#[test]
fn binary_lookup_handles_1024_distinct_logical_positions() {
    let (r, bytes) = patterned();
    let records = (0..1024)
        .map(|i| format!("RW 1 FLAT \"x\" {}\n", i % 16))
        .collect::<String>();
    let d = disk(&descriptor(&records), &r);
    let expected = bytes.repeat(64);
    for offset in (0..expected.len()).step_by(509) {
        let len = (expected.len() - offset).min(1031);
        let mut b = vec![0; len];
        d.read_exact_at(offset as u64, &mut b).unwrap();
        assert_eq!(b, expected[offset..offset + len]);
    }
}
struct Unknown(Arc<dyn BlockDevice>);
impl BlockDevice for Unknown {
    fn geometry(&self) -> DiskGeometry {
        self.0.geometry()
    }
    fn capabilities(&self) -> Capabilities {
        self.0.capabilities()
    }
    fn read_at(&self, o: u64, b: &mut [u8]) -> rvvdk_core::Result<usize> {
        self.0.read_at(o, b)
    }
    fn write_at(&self, _: u64, _: &[u8]) -> rvvdk_core::Result<usize> {
        panic!("no writes")
    }
    fn flush(&self) -> rvvdk_core::Result<()> {
        panic!("no flush")
    }
}
#[test]
fn every_backing_alias_is_checked_and_unknown_identity_copies_fail_closed() {
    struct Two(Source, Source);
    impl rvvdk_vmdk::BackingResolver for Two {
        fn resolve(&self, name: &str) -> Result<Arc<dyn BlockDevice>, rvvdk_vmdk::BackingError> {
            Ok(if name == "x" {
                self.0.0.clone()
            } else {
                self.1.0.clone()
            })
        }
    }
    let (a, _) = patterned();
    let (b, _) = patterned();
    let pair = Two(a, b);
    let d = disk(&descriptor("RW 1 FLAT \"x\" 0\nRW 1 FLAT \"y\" 1"), &pair);
    let destination = AliasTarget(pair.1.0.clone());
    assert!(matches!(
        rvvdk_datamover::Verifier::new(d.size(), 512, 4096)
            .unwrap()
            .verify(&d, &destination),
        Err(Error::AliasedEndpoints)
    ));
    let mover = DataMover::new(CopyOptions::default());
    assert!(matches!(
        mover.copy(&d, &destination),
        Err(Error::AliasedEndpoints)
    ));
    let unknown = Source(Arc::new(Unknown(pair.0.0.clone())));
    let d = disk(&descriptor("RW 1 FLAT \"x\" 0"), &unknown);
    d.read_exact_at(0, &mut [0; 512]).unwrap();
    let destination = RawDisk::new(MemoryBlockDevice::new(512).unwrap());
    assert!(matches!(
        mover.copy(&d, &destination),
        Err(Error::InvalidEndpoint { .. })
    ));
    let zero = disk(&descriptor("RW 1 ZERO"), &unknown);
    zero.validate_destination_identity(CopyEndpoint {
        size: 512,
        capabilities: Capabilities::WRITE,
        identity: None,
    })
    .unwrap();
}
