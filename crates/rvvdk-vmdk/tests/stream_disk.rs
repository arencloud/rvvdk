#[path = "support/stream_disk.rs"]
mod fixture;
use fixture::{bytes, image, stored};
use rvvdk_core::{
    BlockDevice, Capabilities, CopyEndpoint, DiskGeometry, Error, ExtentKind, MemoryBlockDevice,
    VirtualDisk,
};
use rvvdk_vmdk::{StreamDisk, StreamDiskLimits};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
struct Source {
    inner: MemoryBlockDevice,
    reads: Mutex<Vec<(u64, usize)>>,
    changed: AtomicBool,
    partial: bool,
}
impl Source {
    fn new(b: &[u8], partial: bool) -> Arc<Self> {
        let inner = MemoryBlockDevice::new(b.len()).unwrap();
        inner.write_all_at(0, b).unwrap();
        Arc::new(Self {
            inner,
            reads: Mutex::new(vec![]),
            changed: AtomicBool::new(false),
            partial,
        })
    }
}
impl BlockDevice for Source {
    fn geometry(&self) -> DiskGeometry {
        self.inner.geometry()
    }
    fn capabilities(&self) -> Capabilities {
        self.inner.capabilities()
    }
    fn copy_endpoint(&self) -> rvvdk_core::Result<CopyEndpoint> {
        let mut e = self.inner.copy_endpoint()?;
        if self.changed.load(Ordering::Relaxed) {
            e.size -= 512;
        }
        Ok(e)
    }
    fn read_at(&self, at: u64, b: &mut [u8]) -> rvvdk_core::Result<usize> {
        self.reads.lock().unwrap().push((at, b.len()));
        let n = if self.partial {
            b.len().min(37)
        } else {
            b.len()
        };
        self.inner.read_at(at, &mut b[..n])
    }
    fn write_at(&self, at: u64, b: &[u8]) -> rvvdk_core::Result<usize> {
        self.inner.write_at(at, b)
    }
    fn flush(&self) -> rvvdk_core::Result<()> {
        Ok(())
    }
}
fn load(b: &[u8]) -> StreamDisk {
    StreamDisk::load(Source::new(b, false), StreamDiskLimits::default()).unwrap()
}
#[test]
fn native_bytes_ranges_zeros_extents_and_read_only_capabilities() {
    for footer in [false, true] {
        let b = image(
            footer,
            64,
            &[
                (1, stored(&bytes(1))),
                (2, stored(&bytes(2))),
                (63, stored(&bytes(63))),
            ],
        );
        let d = load(&b);
        let mut expected = vec![0; 64 * 65536];
        for i in [1, 2, 63] {
            expected[i * 65536..(i + 1) * 65536].copy_from_slice(&bytes(i as u64));
        }
        let mut all = vec![0xff; expected.len()];
        d.read_exact_at(0, &mut all).unwrap();
        assert_eq!(all, expected);
        for (at, n) in [
            (0, 1),
            (65535, 3),
            (65537, 777),
            (131071, 65540),
            (d.size() - 5, 5),
        ] {
            let mut v = vec![0; n];
            d.read_exact_at(at, &mut v).unwrap();
            assert_eq!(v, expected[at as usize..at as usize + n]);
        }
        let ex = d.extents(65535, 131075).unwrap();
        assert_eq!(
            ex.iter()
                .map(|e| (e.offset(), e.length(), e.kind()))
                .collect::<Vec<_>>(),
            vec![
                (65535, 1, ExtentKind::Zero),
                (65536, 131072, ExtentKind::Data),
                (196608, 2, ExtentKind::Zero)
            ]
        );
        assert!(d.is_read_only());
        assert!(!d.capabilities().contains(Capabilities::WRITE));
        assert!(matches!(d.write_at(0, &[]), Err(Error::Unsupported)));
        assert!(d.write_zero_at(0, 1).is_err());
        assert!(d.discard(0, 1).is_err());
        assert!(d.flush().is_err());
    }
}
#[test]
fn rejects_truncation_checksum_raw_gzip_dictionary_trailing_concatenation_and_wrong_output() {
    let valid = stored(&bytes(1));
    let mut bad_checksum = valid.clone();
    *bad_checksum.last_mut().unwrap() ^= 1;
    let mut trailing = valid.clone();
    trailing.push(0);
    let mut concat = valid.clone();
    concat.extend_from_slice(&stored(&[]));
    let mut dictionary = valid.clone();
    dictionary[..2].copy_from_slice(&[0x78, 0x20]);
    let cases = vec![
        valid[..valid.len() - 1].to_vec(),
        valid[..5].to_vec(),
        bad_checksum,
        trailing,
        concat,
        valid[2..valid.len() - 4].to_vec(),
        vec![0x1f, 0x8b, 8, 0],
        dictionary,
        stored(&bytes(1)[..65535]),
        stored(&vec![0; 65537]),
        stored(&vec![0; 131000]),
    ];
    for payload in cases {
        // Oversized compressed inputs are map-limit failures, not decoder claims.
        let b = image(true, 64, &[(1, payload)]);
        let d = load(&b);
        let mut out = [0xcc; 1];
        assert!(matches!(
            d.read_at(65536, &mut out),
            Err(Error::CorruptMetadata(_))
        ));
        assert_eq!(out, [0xcc]);
    }
}
#[test]
fn cache_hits_failed_replacements_and_revalidation() {
    let b = image(true, 64, &[(1, stored(&bytes(1))), (2, stored(&bytes(2)))]);
    let s = Source::new(&b, false);
    let d = StreamDisk::load(s.clone(), StreamDiskLimits::default()).unwrap();
    s.reads.lock().unwrap().clear();
    let mut out = [0; 9];
    d.read_exact_at(65536, &mut out).unwrap();
    d.read_exact_at(65540, &mut out).unwrap();
    assert_eq!(s.reads.lock().unwrap().len(), 1);
    let g = d.map().grain(2).unwrap().unwrap();
    s.write_all_at(g.payload().offset + g.payload().length - 1, &[0])
        .unwrap();
    assert!(d.read_at(2 * 65536, &mut out).is_err());
    d.read_exact_at(65536, &mut out).unwrap();
    assert_eq!(out, bytes(1)[..9]);
    assert_eq!(s.reads.lock().unwrap().len(), 3);
    d.revalidate().unwrap();
    s.reads.lock().unwrap().clear();
    d.read_exact_at(65536, &mut out).unwrap();
    assert_eq!(s.reads.lock().unwrap().len(), 1);
}
#[test]
fn per_read_limits_precede_io_and_buffer_mutation() {
    let b = image(true, 64, &[(1, stored(&bytes(1)))]);
    for limits in [
        StreamDiskLimits {
            request_bytes: 1,
            ..Default::default()
        },
        StreamDiskLimits {
            encoded_read_bytes: 1,
            ..Default::default()
        },
        StreamDiskLimits {
            decoded_grains: 0,
            ..Default::default()
        },
    ] {
        let s = Source::new(&b, false);
        let d = StreamDisk::load(s.clone(), limits).unwrap();
        s.reads.lock().unwrap().clear();
        let mut out = [0xcc; 2];
        assert!(d.read_at(65536, &mut out).is_err());
        assert_eq!(out, [0xcc; 2]);
        assert!(s.reads.lock().unwrap().is_empty());
    }
    let d = load(&b);
    let required = d.decode_memory_bytes();
    assert!(
        StreamDisk::load(
            Source::new(&b, false),
            StreamDiskLimits {
                decode_memory_bytes: required,
                ..Default::default()
            }
        )
        .is_ok()
    );
    assert!(
        StreamDisk::load(
            Source::new(&b, false),
            StreamDiskLimits {
                decode_memory_bytes: required - 1,
                ..Default::default()
            }
        )
        .is_err()
    );
    let g = d.map().grain(1).unwrap().unwrap();
    let exact = StreamDiskLimits {
        request_bytes: 65536,
        encoded_read_bytes: g.payload().length + 12,
        decoded_grains: 1,
        ..Default::default()
    };
    let d = StreamDisk::load(Source::new(&b, false), exact).unwrap();
    d.read_exact_at(65536, &mut vec![0; 65536]).unwrap();
}
#[test]
fn endpoint_changes_aliases_prefix_mutations_and_partial_backend() {
    let b = image(false, 64, &[(1, stored(&bytes(1)))]);
    let s = Source::new(&b, true);
    let d = StreamDisk::load(s.clone(), StreamDiskLimits::default()).unwrap();
    let mut out = [0; 11];
    d.read_exact_at(65536, &mut out).unwrap();
    assert_eq!(out, bytes(1)[..11]);
    assert!(matches!(
        d.validate_destination_identity(s.copy_endpoint().unwrap()),
        Err(Error::AliasedEndpoints)
    ));
    let other = MemoryBlockDevice::new(512).unwrap();
    d.validate_destination_identity(other.copy_endpoint().unwrap())
        .unwrap();
    assert!(
        d.validate_destination_identity(CopyEndpoint {
            size: 512,
            capabilities: Capabilities::READ,
            identity: None
        })
        .is_err()
    );
    s.changed.store(true, Ordering::Relaxed);
    assert!(matches!(d.revalidate(), Err(Error::EndpointChanged(_))));
    s.changed.store(false, Ordering::Relaxed);
    let g = d.map().grain(1).unwrap().unwrap();
    s.write_all_at(g.marker_offset(), &0u64.to_le_bytes())
        .unwrap();
    d.revalidate().unwrap();
    assert!(matches!(
        d.read_at(65536, &mut out),
        Err(Error::CorruptMetadata(_))
    ));
}
#[test]
fn empty_large_sparse_capacity_and_extent_limits() {
    let b = image(true, 1 << 24, &[]);
    let d = load(&b);
    assert_eq!(d.extents(0, d.size()).unwrap().len(), 1);
    let mut out = [0xff; 11];
    d.read_exact_at(d.size() - 11, &mut out).unwrap();
    assert_eq!(out, [0; 11]);
    assert_eq!(d.read_at(d.size(), &mut []).unwrap(), 0);
    assert!(d.read_at(d.size(), &mut [0]).is_err());
    assert!(d.read_at(u64::MAX, &mut [0]).is_err());
    assert!(d.extents(u64::MAX, 1).is_err());
    assert!(d.extents(d.size(), 0).unwrap().is_empty());
    let b = image(true, 64, &[(1, stored(&bytes(1)))]);
    let s = Source::new(&b, false);
    let d = StreamDisk::load(
        s.clone(),
        StreamDiskLimits {
            output_extents: 2,
            ..Default::default()
        },
    )
    .unwrap();
    s.reads.lock().unwrap().clear();
    assert!(d.extents(0, d.size()).is_err());
    assert!(s.reads.lock().unwrap().is_empty());
}
#[test]
fn concurrent_callers_share_bounded_cache_without_mixing_grains() {
    let b = image(true, 64, &[(1, stored(&bytes(1))), (2, stored(&bytes(2)))]);
    let d = Arc::new(load(&b));
    std::thread::scope(|scope| {
        for i in 0..8 {
            let d = d.clone();
            scope.spawn(move || {
                let index = 1 + i % 2;
                for offset in 0..32 {
                    let mut v = [0; 513];
                    d.read_exact_at(index * 65536 + offset, &mut v).unwrap();
                    assert_eq!(v, bytes(index)[offset as usize..offset as usize + 513]);
                }
            });
        }
    });
}

#[test]
fn independent_zlib_dynamic_codes_zero_grains_and_complete_gzip_rejection() {
    let pattern = include_bytes!("fixtures/stream/pattern.zlib");
    let zero = include_bytes!("fixtures/stream/zero.zlib");
    let d = load(&image(
        true,
        64,
        &[(1, pattern.to_vec()), (2, zero.to_vec())],
    ));
    let mut out = vec![0; 65536];
    d.read_exact_at(65536, &mut out).unwrap();
    assert_eq!(out, bytes(1));
    out.fill(0xaa);
    d.read_exact_at(131072, &mut out).unwrap();
    assert!(out.iter().all(|v| *v == 0));
    assert_eq!(
        d.extents(131072, 65536).unwrap()[0].kind(),
        ExtentKind::Data
    );
    let d = load(&image(
        true,
        64,
        &[(1, include_bytes!("fixtures/stream/pattern.gzip").to_vec())],
    ));
    assert!(d.read_at(65536, &mut out).is_err());
}

#[test]
fn high_ratio_oversized_output_and_deterministic_malformed_streams_are_bounded() {
    let d = load(&image(
        true,
        64,
        &[(1, include_bytes!("fixtures/stream/oversized.zlib").to_vec())],
    ));
    let mut out = [0xa5; 512];
    assert!(d.read_at(65536, &mut out).is_err());
    assert_eq!(out, [0xa5; 512]);
    let mut seed = 0x511b_u64;
    for size in [1, 2, 6, 32, 512, 4096, 131072] {
        let mut payload = vec![0; size];
        for byte in &mut payload {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            *byte = (seed >> 32) as u8;
        }
        let d = load(&image(true, 64, &[(1, payload)]));
        assert!(d.read_at(65536, &mut out).is_err());
        assert_eq!(out, [0xa5; 512]);
    }
}
