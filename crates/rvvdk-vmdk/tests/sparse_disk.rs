#[path = "support/sparse_metadata.rs"]
mod f;
use rvvdk_core::{
    BlockDevice, Capabilities, CopyEndpoint, DiskGeometry, EndpointIdentity, Error, ExtentKind,
    MemoryBlockDevice, RawDisk, VirtualDisk,
};
use rvvdk_datamover::{CopyOptions, DataMover, Verifier};
use rvvdk_vmdk::{
    BackingError, BackingResolver, SparseDescriptor, SparseDisk, SparseDiskLimits, SparseMetadata,
    SparseMetadataError,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
const M: usize = 1048576;
const G: usize = 65536;
fn text() -> String {
    f::descriptor("monolithicSparse", "RW 2048 SPARSE \"disk.vmdk\"")
}
fn disk(t: &str, r: &dyn BackingResolver) -> SparseDisk {
    SparseDisk::load(
        &SparseDescriptor::parse(t.as_bytes()).unwrap(),
        r,
        SparseDiskLimits::default(),
    )
    .unwrap()
}
fn oracle() -> Vec<u8> {
    let mut b = vec![0; M];
    b[..G].fill(0x5a);
    b[G..2 * G].fill(0xa5);
    b
}
struct Source {
    device: Arc<MemoryBlockDevice>,
    mode: AtomicUsize,
    reads: AtomicUsize,
}
impl Source {
    fn new(b: &[u8]) -> Arc<Self> {
        Arc::new(Self {
            device: f::device(b),
            mode: AtomicUsize::new(0),
            reads: AtomicUsize::new(0),
        })
    }
}
impl BlockDevice for Source {
    fn geometry(&self) -> DiskGeometry {
        self.device.geometry()
    }
    fn capabilities(&self) -> Capabilities {
        if self.mode.load(Ordering::Relaxed) & 16 != 0 {
            Capabilities::empty()
        } else {
            Capabilities::READ
        }
    }
    fn copy_endpoint(&self) -> rvvdk_core::Result<CopyEndpoint> {
        let mode = self.mode.load(Ordering::Relaxed);
        Ok(CopyEndpoint {
            size: if mode & 4 != 0 { 0 } else { self.device.size() },
            capabilities: self.capabilities(),
            identity: if mode & 32 != 0 {
                None
            } else if mode & 8 != 0 {
                Some(EndpointIdentity::Memory {
                    address: usize::MAX,
                })
            } else {
                self.device.copy_endpoint()?.identity
            },
        })
    }
    fn read_at(&self, o: u64, b: &mut [u8]) -> rvvdk_core::Result<usize> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        let mode = self.mode.load(Ordering::Relaxed);
        if o >= G as u64 {
            if mode & 1 != 0 {
                return Ok(0);
            }
            if mode & 2 != 0 {
                return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied).into());
            }
            if mode & 64 != 0 {
                let n = b.len().min(7);
                return self.device.read_at(o, &mut b[..n]);
            }
        }
        self.device.read_at(o, b)
    }
    fn write_at(&self, _: u64, _: &[u8]) -> rvvdk_core::Result<usize> {
        panic!("source write")
    }
    fn flush(&self) -> rvvdk_core::Result<()> {
        panic!("source flush")
    }
}
struct Pair {
    a: Arc<Source>,
    b: Arc<Source>,
    opens: AtomicUsize,
}
impl BackingResolver for Pair {
    fn resolve(&self, name: &str) -> Result<Arc<dyn BlockDevice>, BackingError> {
        self.opens.fetch_add(1, Ordering::Relaxed);
        Ok(if name == "a" {
            self.a.clone()
        } else {
            self.b.clone()
        })
    }
}
fn pair() -> (String, Pair) {
    (
        f::descriptor(
            "twoGbMaxExtentSparse",
            "RW 2048 SPARSE \"a\"\nRDONLY 2048 SPARSE \"b\"",
        ),
        Pair {
            a: Source::new(&f::bytes(None)),
            b: Source::new(&f::bytes(None)),
            opens: AtomicUsize::new(0),
        },
    )
}
#[test]
fn reads_match_oracle_at_grain_boundaries_and_unaligned_ranges() {
    let t = text();
    let d = disk(&t, &f::Resolver(f::device(&f::bytes(Some(&t)))));
    let expected = oracle();
    for offset in (0..=M)
        .step_by(509)
        .chain([G - 1, G, G + 1, 2 * G - 1, 2 * G, M])
    {
        for len in [0, 1, 3, 4097, 65537, M - offset] {
            if offset + len > M {
                continue;
            }
            let mut b = vec![0xcc; len];
            assert_eq!(d.read_at(offset as u64, &mut b).unwrap(), len);
            assert_eq!(b, expected[offset..offset + len]);
        }
    }
    assert_eq!(d.cid(), 0x12345678);
    assert_eq!(d.geometry().logical_block_size(), 512);
    assert_eq!(d.copy_endpoint().unwrap().identity, None);
    assert!(d.is_read_only());
}
#[test]
fn contiguous_grains_merge_reads_and_zeros_never_read_payload() {
    let t = text();
    let s = Source::new(&f::bytes(Some(&t)));
    let d = disk(&t, &f::Resolver(s.clone()));
    s.reads.store(0, Ordering::Relaxed);
    let mut b = vec![0; 2 * G];
    d.read_exact_at(0, &mut b).unwrap();
    assert_eq!(s.reads.load(Ordering::Relaxed), 1);
    assert_eq!(b, oracle()[..2 * G]);
    d.read_exact_at((2 * G) as u64, &mut b).unwrap();
    assert_eq!(b, vec![0; 2 * G]);
    assert_eq!(s.reads.load(Ordering::Relaxed), 1);
    let mut raw = f::bytes(Some(&t));
    for table in [22, 27] {
        f::put32(&mut raw, table * 512, 256);
        f::put32(&mut raw, table * 512 + 4, 128);
    }
    let s = Source::new(&raw);
    let d = disk(&t, &f::Resolver(s.clone()));
    s.reads.store(0, Ordering::Relaxed);
    d.read_exact_at(0, &mut b).unwrap();
    assert_eq!(s.reads.load(Ordering::Relaxed), 2);
    assert_eq!(&b[..G], vec![0xa5; G]);
    assert_eq!(&b[G..], vec![0x5a; G]);
    assert_eq!(d.extents(0, (2 * G) as u64).unwrap().len(), 1);
}
#[test]
fn split_reads_and_extents_cross_backings_with_different_grain_sizes() {
    let (t, p) = pair();
    let mut second = f::bytes(None);
    second[20..28].copy_from_slice(&64u64.to_le_bytes()); // 32 KiB grains
    p.b.device.write_all_at(0, &second).unwrap();
    let d = disk(&t, &p);
    let mut expected = oracle();
    let mut tail = vec![0; M];
    tail[..G / 2].fill(0x5a);
    tail[G / 2..G].fill(0xa5);
    expected.extend(tail);
    for offset in [M - G, M - 3, M, M + G / 2 - 1, 2 * M - 1] {
        let length = (2 * M - offset).min(G + 31);
        let mut b = vec![9; length];
        d.read_exact_at(offset as u64, &mut b).unwrap();
        assert_eq!(b, expected[offset..offset + length]);
    }
    let e = d.extents((M - 17) as u64, (G + 35) as u64).unwrap();
    assert_eq!(
        e.iter()
            .map(|e| (e.offset(), e.length(), e.kind()))
            .collect::<Vec<_>>(),
        vec![
            ((M - 17) as u64, 17, ExtentKind::Zero),
            (M as u64, G as u64, ExtentKind::Data),
            ((M + G) as u64, 18, ExtentKind::Zero)
        ]
    );
    assert_eq!(p.opens.load(Ordering::Relaxed), 2);
}
#[test]
fn extent_queries_clip_coalesce_across_sources_and_enforce_output_limit() {
    let (t, p) = pair();
    for table in [22, 27] {
        p.b.device
            .write_all_at((table * 512) as u64, &0u32.to_le_bytes())
            .unwrap();
    }
    let d = disk(&t, &p);
    let e = d
        .extents((2 * G - 7) as u64, (M + G + 10 - 2 * G + 7) as u64)
        .unwrap();
    assert_eq!(
        e.iter()
            .map(|e| (e.offset(), e.length(), e.kind()))
            .collect::<Vec<_>>(),
        vec![
            ((2 * G - 7) as u64, 7, ExtentKind::Data),
            ((2 * G) as u64, (M - G) as u64, ExtentKind::Zero),
            ((M + G) as u64, 10, ExtentKind::Data)
        ]
    );
    let d = SparseDisk::load(
        &SparseDescriptor::parse(t.as_bytes()).unwrap(),
        &p,
        SparseDiskLimits {
            output_extents: 1,
            ..SparseDiskLimits::default()
        },
    )
    .unwrap();
    assert_eq!(d.extents(17, 33).unwrap().len(), 1);
    assert!(matches!(
        d.extents(0, 2 * M as u64),
        Err(Error::InvalidEndpoint { .. })
    ));
    assert!(d.extents(2 * M as u64, 0).unwrap().is_empty());
}
#[test]
fn aggregate_budgets_reject_before_later_offset_reads_and_release_handles() {
    let (t, p) = pair();
    let parsed = SparseDescriptor::parse(t.as_bytes()).unwrap();
    let d = disk(&t, &p);
    let memory = d.reserved_memory_bytes();
    let reads = d.metadata_read_bytes();
    assert_eq!(
        memory,
        2 * 11488 + 2 * std::mem::size_of::<SparseMetadata>() as u64
    );
    assert_eq!(reads, 32768);
    drop(d);
    let exact = SparseDiskLimits {
        memory_bytes: memory,
        read_bytes: reads,
        ..SparseDiskLimits::default()
    };
    assert!(SparseDisk::load(&parsed, &p, exact).is_ok());
    for limits in [
        SparseDiskLimits {
            memory_bytes: memory - 1,
            ..exact
        },
        SparseDiskLimits {
            read_bytes: reads - 1,
            ..exact
        },
    ] {
        p.b.reads.store(0, Ordering::Relaxed);
        assert!(matches!(
            SparseDisk::load(&parsed, &p, limits),
            Err(SparseMetadataError::Limit(_))
        ));
        assert_eq!(p.b.reads.load(Ordering::Relaxed), 1);
        assert_eq!(Arc::strong_count(&p.a), 1);
        assert_eq!(Arc::strong_count(&p.b), 1);
    }
    p.opens.store(0, Ordering::Relaxed);
    for limits in [
        SparseDiskLimits {
            extents: 1,
            ..exact
        },
        SparseDiskLimits {
            memory_bytes: 0,
            ..exact
        },
        SparseDiskLimits {
            read_bytes: 511,
            ..exact
        },
    ] {
        assert!(SparseDisk::load(&parsed, &p, limits).is_err());
        assert_eq!(p.opens.load(Ordering::Relaxed), 0);
    }
}
#[test]
fn invalid_ranges_do_not_touch_buffers_and_all_mutations_reject() {
    let t = text();
    let d = disk(&t, &f::Resolver(f::device(&f::bytes(Some(&t)))));
    for o in [M as u64 - 1, M as u64 + 1, u64::MAX] {
        let mut b = [19; 2];
        assert!(d.read_at(o, &mut b).is_err());
        assert_eq!(b, [19; 2]);
    }
    assert_eq!(d.read_at(M as u64, &mut []).unwrap(), 0);
    assert!(d.read_at(M as u64 + 1, &mut []).is_err());
    assert!(matches!(
        d.extents(u64::MAX, 1),
        Err(Error::RangeOverflow { .. })
    ));
    assert!(matches!(d.write_at(0, &[0]), Err(Error::Unsupported)));
    assert!(matches!(d.write_zero_at(0, 1), Err(Error::Unsupported)));
    assert!(matches!(d.discard(0, 1), Err(Error::Unsupported)));
    assert!(matches!(d.flush(), Err(Error::Unsupported)));
    assert!(
        !d.capabilities()
            .intersects(Capabilities::WRITE | Capabilities::DIRECT_IO | Capabilities::FLUSH)
    );
}
#[test]
fn short_reads_complete_and_payload_eof_or_errors_are_never_zero_filled() {
    let t = text();
    let s = Source::new(&f::bytes(Some(&t)));
    let d = disk(&t, &f::Resolver(s.clone()));
    s.mode.store(64, Ordering::Relaxed);
    let mut b = [9; 33];
    d.read_exact_at(G as u64 - 17, &mut b).unwrap();
    assert_eq!(&b[..17], &[0x5a; 17]);
    assert_eq!(&b[17..], &[0xa5; 16]);
    for mode in [1, 2] {
        s.mode.store(mode, Ordering::Relaxed);
        b.fill(9);
        let e = d.read_at(0, &mut b).unwrap_err();
        assert!(matches!(
            (mode, e),
            (1, Error::UnexpectedEof { .. }) | (2, Error::Io(_))
        ));
        assert_eq!(b, [9; 33]);
    }
}
#[test]
fn retained_sources_support_concurrent_reads_after_input_and_resolver_drop() {
    let d = {
        let t = text();
        let r = f::Resolver(f::device(&f::bytes(Some(&t))));
        disk(&t, &r)
    };
    std::thread::scope(|scope| {
        for _ in 0..4 {
            let d = &d;
            scope.spawn(move || {
                for _ in 0..25 {
                    let mut b = [0; 513];
                    d.read_exact_at(2 * G as u64 - 257, &mut b).unwrap();
                    assert_eq!(&b[..257], &[0xa5; 257]);
                    assert_eq!(&b[257..], &[0; 256]);
                }
            });
        }
    });
}
#[test]
fn portable_copy_and_verify_preserve_zeros_and_destination_tail() {
    let (t, p) = pair();
    let d = disk(&t, &p);
    let expected = oracle().repeat(2);
    for workers in [1, 4] {
        let target = RawDisk::new(MemoryBlockDevice::new(2 * M + 512).unwrap());
        target.write_all_at(0, &vec![0xcc; 2 * M + 512]).unwrap();
        let mover = DataMover::new(CopyOptions::with_concurrency(65536, 4096, workers).unwrap());
        let plan = mover.plan_with_destination(&d, &target).unwrap();
        mover.execute_plan(&plan, &d, &target).unwrap();
        let mut actual = vec![0; 2 * M + 512];
        target.read_exact_at(0, &mut actual).unwrap();
        assert_eq!(&actual[..2 * M], expected);
        assert_eq!(&actual[2 * M..], &[0xcc; 512]);
        Verifier::new(d.size(), 65537, 131074)
            .unwrap()
            .verify(&d, &target)
            .unwrap();
    }
}
struct Alias(CopyEndpoint);
impl VirtualDisk for Alias {
    fn geometry(&self) -> DiskGeometry {
        DiskGeometry::new(2 * M as u64, 512, 512).unwrap()
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities::READ | Capabilities::WRITE | Capabilities::FLUSH
    }
    fn copy_endpoint(&self) -> rvvdk_core::Result<CopyEndpoint> {
        Ok(CopyEndpoint {
            size: self.size(),
            capabilities: self.capabilities(),
            identity: self.0.identity,
        })
    }
    fn read_at(&self, _: u64, _: &mut [u8]) -> rvvdk_core::Result<usize> {
        panic!("alias read")
    }
    fn write_at(&self, _: u64, _: &[u8]) -> rvvdk_core::Result<usize> {
        panic!("alias write")
    }
    fn flush(&self) -> rvvdk_core::Result<()> {
        panic!("alias flush")
    }
    fn write_zero_at(&self, _: u64, _: u64) -> rvvdk_core::Result<()> {
        panic!("alias zero")
    }
    fn discard(&self, _: u64, _: u64) -> rvvdk_core::Result<()> {
        panic!("alias discard")
    }
    fn extents(&self, _: u64, _: u64) -> rvvdk_core::Result<Vec<rvvdk_core::Extent>> {
        Err(Error::Unsupported)
    }
}
#[test]
fn every_backing_alias_and_unknown_identity_fail_copy_and_verify_preflight() {
    let (t, p) = pair();
    let d = disk(&t, &p);
    let mover = DataMover::new(CopyOptions::default());
    let plan = mover.plan(&d).unwrap();
    for s in [&p.a, &p.b] {
        let target = Alias(s.copy_endpoint().unwrap());
        assert!(matches!(
            mover.copy(&d, &target),
            Err(Error::AliasedEndpoints)
        ));
        assert!(matches!(
            mover.plan_with_destination(&d, &target),
            Err(Error::AliasedEndpoints)
        ));
        let v: &dyn VirtualDisk = &d;
        assert!(matches!(
            mover.execute_plan(&plan, v, &target),
            Err(Error::AliasedEndpoints)
        ));
        assert!(matches!(
            Verifier::new(d.size(), 512, 4096)
                .unwrap()
                .verify(&d, &target),
            Err(Error::AliasedEndpoints)
        ));
    }
    let unknown = Alias(CopyEndpoint {
        size: 0,
        capabilities: Capabilities::empty(),
        identity: None,
    });
    assert!(matches!(
        d.validate_destination_identity(unknown.copy_endpoint().unwrap()),
        Err(Error::InvalidEndpoint { .. })
    ));
    p.b.mode.store(32, Ordering::Relaxed);
    let d = disk(&t, &p);
    d.read_exact_at(M as u64, &mut [0; 1]).unwrap();
    let target = RawDisk::new(MemoryBlockDevice::new(2 * M).unwrap());
    assert!(matches!(
        mover.copy(&d, &target),
        Err(Error::InvalidEndpoint { .. })
    ));
}
#[test]
fn endpoint_changes_after_planning_reject_before_destination_writes() {
    let (t, p) = pair();
    let d = disk(&t, &p);
    let mover = DataMover::new(CopyOptions::default());
    let plan = mover.plan(&d).unwrap();
    let target = RawDisk::new(MemoryBlockDevice::new(2 * M).unwrap());
    target.write_all_at(0, &vec![41; 2 * M]).unwrap();
    for mode in [4, 8, 16, 32] {
        p.a.mode.store(mode, Ordering::Relaxed);
        assert!(d.revalidate().is_err());
        assert!(mover.execute_plan(&plan, &d, &target).is_err());
        let mut b = [0; 1];
        target.read_exact_at(0, &mut b).unwrap();
        assert_eq!(b, [41]);
    }
}
#[test]
fn final_acquisition_rechecks_earlier_sources() {
    struct Changing(Pair);
    impl BackingResolver for Changing {
        fn resolve(&self, name: &str) -> Result<Arc<dyn BlockDevice>, BackingError> {
            if name == "b" {
                self.0.a.mode.store(8, Ordering::Relaxed);
            }
            self.0.resolve(name)
        }
    }
    let (t, p) = pair();
    assert!(
        SparseDisk::load(
            &SparseDescriptor::parse(t.as_bytes()).unwrap(),
            &Changing(p),
            SparseDiskLimits::default()
        )
        .is_err()
    );
}

#[cfg(target_os = "linux")]
#[test]
fn local_hardlink_alias_and_path_replacement_use_retained_inode() {
    use rvvdk_vmdk::LocalResolver;
    use std::{
        fs::{self, File},
        path::PathBuf,
    };
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    let root = std::env::var_os("RVVDK_TEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let directory = Cleanup(root.join(format!("rvvdk-r53-{}", std::process::id())));
    fs::create_dir(&directory.0).unwrap();
    let t = text();
    let raw = f::bytes(Some(&t));
    fs::write(directory.0.join("disk.vmdk"), &raw).unwrap();
    fs::hard_link(directory.0.join("disk.vmdk"), directory.0.join("alias")).unwrap();
    let resolver = LocalResolver::from_directory(File::open(&directory.0).unwrap()).unwrap();
    let d = disk(&t, &resolver);
    let alias = resolver.resolve("alias").unwrap();
    assert!(matches!(
        d.validate_destination_identity(alias.copy_endpoint().unwrap()),
        Err(Error::AliasedEndpoints)
    ));
    fs::rename(directory.0.join("disk.vmdk"), directory.0.join("retained")).unwrap();
    fs::write(directory.0.join("disk.vmdk"), b"replacement").unwrap();
    d.revalidate().unwrap();
    let mut b = vec![0; M];
    d.read_exact_at(0, &mut b).unwrap();
    assert_eq!(b, oracle());
    assert_eq!(fs::read(directory.0.join("retained")).unwrap(), raw);
}
