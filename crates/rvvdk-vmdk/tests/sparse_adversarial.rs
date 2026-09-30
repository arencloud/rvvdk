#[path = "support/sparse_scale.rs"]
#[allow(dead_code)]
mod f;
use f::{Device, Layout, Pattern, Resolver, put32};
use rvvdk_core::{Error, ExtentKind, VirtualDisk};
use rvvdk_vmdk::{SparseDescriptor, SparseDisk, SparseDiskLimits, SparseMetadataError};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
fn load(source: Arc<Device>, limits: SparseDiskLimits) -> Result<SparseDisk, SparseMetadataError> {
    let text = source.layout.text.clone();
    SparseDisk::load(
        &SparseDescriptor::parse(text.as_bytes()).unwrap(),
        &Resolver(source),
        limits,
    )
}
fn next(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}
#[test]
fn generated_geometries_and_table_boundaries_match_independent_byte_oracles() {
    let mut seed = 0x52563535u64;
    for grain in [8192, 65536, 1048576] {
        for count in [17, 511, 512, 513, 1024] {
            for pattern in [
                Pattern::Zero,
                Pattern::Contiguous,
                Pattern::Reversed,
                Pattern::Alternating,
            ] {
                let layout = Layout::new(count * grain, grain);
                let d = load(
                    Arc::new(Device::new(layout.clone(), pattern, false)),
                    SparseDiskLimits::default(),
                )
                .unwrap();
                for offset in [0, grain - 1, grain, layout.capacity - 1]
                    .into_iter()
                    .chain((0..20).map(|_| next(&mut seed) % layout.capacity))
                {
                    let len = (layout.capacity - offset).min(65537) as usize;
                    let mut actual = vec![0xcc; len];
                    d.read_exact_at(offset, &mut actual).unwrap();
                    assert_eq!(
                        actual,
                        layout.expected(offset, len, pattern),
                        "{grain} {count} {pattern:?} {offset}"
                    );
                    let extents = d.extents(offset, len as u64).unwrap();
                    let mut pos = offset;
                    for e in extents {
                        assert_eq!(e.offset(), pos);
                        assert!(e.length() > 0);
                        for byte in [e.offset(), e.end() - 1] {
                            assert_eq!(
                                e.kind(),
                                if layout.slot(byte / grain, pattern).is_some() {
                                    ExtentKind::Data
                                } else {
                                    ExtentKind::Zero
                                }
                            );
                        }
                        pos = e.end();
                    }
                    assert_eq!(pos, offset + len as u64);
                }
            }
        }
    }
}
#[test]
fn coalescing_and_fragmentation_have_exact_backend_request_counts() {
    for (pattern, calls) in [
        (Pattern::Zero, 0),
        (Pattern::Contiguous, 1),
        (Pattern::Reversed, 16),
        (Pattern::Permuted, 16),
        (Pattern::Alternating, 8),
    ] {
        let l = Layout::new(1024 * 65536, 65536);
        let s = Arc::new(Device::new(l.clone(), pattern, true));
        let d = load(s.clone(), SparseDiskLimits::default()).unwrap();
        s.reset();
        let mut b = vec![0; 16 * 65536];
        d.read_exact_at(65536, &mut b).unwrap();
        assert_eq!(s.calls.load(Ordering::Relaxed), calls);
        assert_eq!(b, l.expected(65536, b.len(), pattern));
    }
}
#[test]
fn metadata_mutation_corpus_is_bounded_and_admitted_maps_match_encoded_tables() {
    let layout = Layout::new(513 * 65536, 65536);
    let original = layout.metadata(Pattern::Alternating);
    let mut seed = 0x617576355u64;
    let mut admitted = 0;
    let mut rejected = 0;
    let limits = SparseDiskLimits {
        memory_bytes: 128 * 1024,
        read_bytes: 128 * 1024,
        ..SparseDiskLimits::default()
    };
    for case in 0..4096 {
        let mut s = Device::new(layout.clone(), Pattern::Alternating, true);
        s.metadata.clone_from(&original);
        s.ceiling = limits.read_bytes;
        let (base, len) = match case % 5 {
            0 => (0, 512),
            1 => (512, layout.text.len() + 8),
            2 => (layout.gd, layout.directory_bytes),
            3 => (layout.rgt, layout.tables * 2048),
            _ => (layout.gt, layout.tables * 2048),
        };
        let pos = base + (next(&mut seed) as usize % len);
        if case % 64 == 0 {
            let at = 512 + layout.text.find("parentCID=ffffffff").unwrap() + "parentCID=".len();
            s.metadata[at] = b'F'; // semantic-preserving positive control
        } else {
            s.metadata[pos] ^= (next(&mut seed) as u8) | 1;
        }
        let s = Arc::new(s);
        match load(s.clone(), limits) {
            Err(_) => rejected += 1,
            Ok(d) => {
                admitted += 1;
                assert!(d.reserved_memory_bytes() <= limits.memory_bytes);
                assert!(s.bytes.load(Ordering::Relaxed) <= limits.read_bytes);
                // Decode admitted primary table words independently from source bytes.
                // Geometry changes still bind to the caller's capacity; use the admitted
                // header solely to locate bounded input words, not to compute the map.
                let h = d.metadata()[0].header();
                let gd = h.primary_directory().offset() as usize;
                for (i, &entry) in d.metadata()[0].grain_sectors().iter().enumerate() {
                    let pointer = u32::from_le_bytes(
                        s.metadata[gd + i / 512 * 4..gd + i / 512 * 4 + 4]
                            .try_into()
                            .unwrap(),
                    ) as usize
                        * 512;
                    assert_eq!(
                        entry,
                        u32::from_le_bytes(
                            s.metadata[pointer + i % 512 * 4..pointer + i % 512 * 4 + 4]
                                .try_into()
                                .unwrap()
                        )
                    );
                }
                let e = d.extents(0, d.size()).unwrap();
                assert_eq!(e.first().unwrap().offset(), 0);
                assert_eq!(e.last().unwrap().end(), d.size());
            }
        }
    }
    assert!(admitted > 0 && rejected > 0);
    println!("corpus seed=0x617576355 cases=4096 admitted={admitted} rejected={rejected}");
}
#[test]
fn invalid_directory_and_grain_pointer_sentinels_never_escape_source_bounds() {
    let l = Layout::new(513 * 65536, 65536);
    for value in [0, 1, 2, 21, 22, u32::MAX] {
        let mut s = Device::new(l.clone(), Pattern::Contiguous, true);
        put32(&mut s.metadata, l.gd + 4, value);
        let s = Arc::new(s);
        assert!(load(s.clone(), SparseDiskLimits::default()).is_err());
        assert_eq!(s.calls.load(Ordering::Relaxed), 4);
    }
    for value in [1, 2, 127, u32::MAX] {
        let mut s = Device::new(l.clone(), Pattern::Contiguous, true);
        for base in [l.gt, l.rgt] {
            put32(&mut s.metadata, base + 512 * 4, value);
        }
        assert!(load(Arc::new(s), SparseDiskLimits::default()).is_err());
    }
    // A cross-table duplicate is invalid even when both redundant copies agree.
    let mut s = Device::new(l.clone(), Pattern::Contiguous, true);
    for base in [l.gt, l.rgt] {
        put32(&mut s.metadata, base + 512 * 4, (l.overhead / 512) as u32);
    }
    assert!(load(Arc::new(s), SparseDiskLimits::default()).is_err());
}
#[test]
fn truncation_at_every_metadata_region_boundary_is_rejected() {
    let l = Layout::new(513 * 65536, 65536);
    for size in [
        0,
        511,
        512,
        10751,
        l.gd as u64 - 1,
        l.rgt as u64,
        l.gt as u64,
        l.overhead - 1,
        l.overhead,
        l.overhead + l.capacity - 1,
    ] {
        let mut s = Device::new(l.clone(), Pattern::Contiguous, false);
        s.observed_size = size;

        assert!(
            load(Arc::new(s), SparseDiskLimits::default()).is_err(),
            "size={size}"
        );
    }
}
#[test]
fn maximum_header_capacity_is_rejected_by_aggregate_budget_before_offset_reads() {
    let l = Layout::new(1 << 40, 65536);
    let s = Arc::new(Device {
        metadata: l.header().to_vec(),
        observed_size: l.overhead + l.capacity,
        layout: l,
        track: true,
        calls: AtomicU64::new(0),
        bytes: AtomicU64::new(0),
        ceiling: 512,
    });
    assert!(matches!(
        load(s.clone(), SparseDiskLimits::default()),
        Err(SparseMetadataError::Limit(_))
    ));
    assert_eq!(s.calls.load(Ordering::Relaxed), 1);
}
#[test]
fn output_count_boundary_and_small_queries_do_not_require_whole_disk_output() {
    for count in [65536, 65537] {
        let l = Layout::new(count * 8192, 8192);
        let s = Arc::new(Device::new(l.clone(), Pattern::Alternating, true));
        let d = load(s.clone(), SparseDiskLimits::default()).unwrap();
        s.reset();
        let result = d.extents(0, d.size());
        if count == 65536 {
            assert_eq!(result.unwrap().len(), 65536)
        } else {
            assert!(matches!(result, Err(Error::InvalidEndpoint { .. })));
        }
        let e = d.extents(d.size() - 1, 1).unwrap();
        assert_eq!(e.len(), 1);
        assert_eq!(
            e[0].kind(),
            if count % 2 == 1 {
                ExtentKind::Data
            } else {
                ExtentKind::Zero
            }
        );
        assert_eq!(s.calls.load(Ordering::Relaxed), 0);
    }
}

#[test]
fn capacity_profiles_remain_within_default_admission_and_report_one_zero_extent() {
    for capacity in [1 << 20, 1 << 30, 64 << 30] {
        let l = Layout::new(capacity, 65536);
        let source = Arc::new(Device::new(l.clone(), Pattern::Zero, true));
        let d = load(source.clone(), SparseDiskLimits::default()).unwrap();
        assert_eq!(
            source.bytes.load(Ordering::Relaxed),
            d.metadata_read_bytes()
        );
        let mut b = [7; 4096];
        d.read_exact_at(capacity - 4096, &mut b).unwrap();
        assert_eq!(b, [0; 4096]);
        let e = d.extents(0, d.size()).unwrap();
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].kind(), ExtentKind::Zero);
        println!(
            "R55_PROFILE {{\"capacity_bytes\":{},\"grain_bytes\":65536,\"grain_count\":{},\"tables_per_copy\":{},\"fixture_metadata_bytes\":{},\"reserved_memory_bytes\":{},\"metadata_read_bytes\":{},\"backend_calls\":{}}}",
            capacity,
            l.grains,
            l.tables,
            l.overhead,
            d.reserved_memory_bytes(),
            d.metadata_read_bytes(),
            source.calls.load(Ordering::Relaxed)
        );
    }
}
