#[path = "support/chain_disk.rs"]
#[allow(dead_code)]
mod f;
use f::{Fixture, Spec};
use rvvdk_core::{
    BlockDevice, Capabilities, Error, ExtentKind, MemoryBlockDevice, RawDisk, VirtualDisk,
};
use rvvdk_datamover::{CopyOptions, DataMover, Verifier};
use rvvdk_vmdk::{SparseChainDisk, SparseChainDiskLimits};
use std::sync::atomic::Ordering;
const SIZE: u64 = 262144;
fn mixed() -> Fixture {
    Fixture::new(
        vec![
            Spec::new(vec![SIZE / 2, SIZE / 2], 8192, 3, 1, 11),
            Spec::new(vec![SIZE], 65536, 2, 0, 71),
            Spec::new(vec![SIZE], 16384, 2, 1, 131),
        ],
        true,
    )
}
#[test]
fn different_grains_and_extent_boundaries_match_independent_oracle() {
    for child in [8192, 16384, 65536] {
        for parent in [8192, 32768, 65536] {
            let mut b = Spec::new(vec![SIZE], parent, 2, 1, 173);
            b.reverse = true;
            let f = Fixture::new(
                vec![Spec::new(vec![SIZE / 2, SIZE / 2], child, 3, 1, 13), b],
                true,
            );
            let d = f.disk();
            let mut seed = 0x525637u64;
            for offset in [0, 1, child - 1, parent - 1, SIZE / 2 - 1, SIZE - 1]
                .into_iter()
                .chain((0..32).map(|_| {
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    seed % SIZE
                }))
            {
                let len = (SIZE - offset).min(65537) as usize;
                let mut bytes = vec![0; len];
                d.read_exact_at(offset, &mut bytes).unwrap();
                assert_eq!(bytes, f.oracle(offset, len));
                let e = d.extents(offset, len as u64).unwrap();
                let mut next = offset;
                for entry in e {
                    assert_eq!(entry.offset(), next);
                    assert!(entry.length() > 0);
                    for pos in entry.offset()..entry.end() {
                        let data = f
                            .specs
                            .iter()
                            .any(|s| s.allocated[(pos / s.grain) as usize]);
                        assert_eq!(
                            entry.kind(),
                            if data {
                                ExtentKind::Data
                            } else {
                                ExtentKind::Zero
                            }
                        );
                    }
                    next = entry.end();
                }
                assert_eq!(next, offset + len as u64);
            }
        }
    }
}
#[test]
fn leaf_overrides_parent_and_zero_allocated_bytes_do_not_fall_through() {
    let f = Fixture::new(
        vec![
            Spec::new(vec![SIZE], 8192, 2, 1, 1),
            Spec::new(vec![SIZE], 65536, 1, 0, 99),
        ],
        true,
    );
    let d = f.disk();
    let map = &d.chain().layers()[0].metadata()[0];
    let physical = u64::from(map.grain_sectors()[1]) * 512;
    f.layers[0].backings[0]
        .inner
        .write_all_at(physical, &[0; 8192])
        .unwrap();
    let mut b = vec![9; 16384];
    d.read_exact_at(0, &mut b).unwrap();
    assert_eq!(&b[..8192], &f.oracle(0, 8192));
    assert_eq!(&b[8192..], &[0; 8192]);
    assert_eq!(d.extents(0, 16384).unwrap()[0].kind(), ExtentKind::Data);
}
#[test]
fn sixteen_layer_fallback_zero_and_leaf_short_circuit() {
    for depth in [1, 4, 16] {
        for base in [false, true] {
            let mut specs = vec![Spec::new(vec![SIZE], 8192, 0, 0, 1); depth];
            if base {
                specs[depth - 1] = Spec::new(vec![SIZE], 65536, 1, 0, 101);
            }
            let f = Fixture::new(specs, true);
            let d = f.disk();
            f.reset();
            let mut bytes = vec![9; SIZE as usize];
            d.read_exact_at(0, &mut bytes).unwrap();
            assert_eq!(bytes, f.oracle(0, bytes.len()));
            assert_eq!(f.reads(), usize::from(base));
        }
    }
    let f = Fixture::new(
        vec![
            Spec::new(vec![SIZE], 8192, 1, 0, 1),
            Spec::new(vec![SIZE], 65536, 1, 0, 99),
        ],
        true,
    );
    let d = f.disk();
    f.reset();
    f.layers[1].backings[0].mode.store(2, Ordering::Relaxed);
    d.read_exact_at(0, &mut [0; 8192]).unwrap();
    assert_eq!(f.layers[1].backings[0].calls.load(Ordering::Relaxed), 0);
}
#[test]
fn coalesces_only_physically_adjacent_runs_in_one_backing() {
    for reverse in [false, true] {
        let mut s = Spec::new(vec![SIZE], 8192, 1, 0, 10);
        s.reverse = reverse;
        let f = Fixture::new(vec![s], true);
        let d = f.disk();
        f.reset();
        let mut b = vec![0; SIZE as usize];
        d.read_exact_at(0, &mut b).unwrap();
        assert_eq!(b, f.oracle(0, b.len()));
        assert_eq!(f.reads(), if reverse { 32 } else { 1 });
        assert_eq!(d.extents(0, SIZE).unwrap().len(), 1);
    }
    let f = Fixture::new(
        vec![Spec::new(vec![SIZE / 2, SIZE / 2], 8192, 1, 0, 1)],
        true,
    );
    let d = f.disk();
    f.reset();
    d.read_exact_at(0, &mut vec![0; SIZE as usize]).unwrap();
    assert_eq!(f.reads(), 2);
    assert_eq!(d.extents(0, SIZE).unwrap().len(), 1);
}
#[test]
fn range_errors_and_empty_operations_do_not_touch_backings() {
    let f = mixed();
    let d = f.disk();
    f.reset();
    assert_eq!(d.read_at(SIZE, &mut []).unwrap(), 0);
    assert!(d.extents(SIZE, 0).unwrap().is_empty());
    assert!(d.read_at(SIZE, &mut [0]).is_err());
    assert!(d.read_at(SIZE + 1, &mut []).is_err());
    assert!(d.extents(u64::MAX, 2).is_err());
    assert_eq!(f.reads(), 0);
}
#[test]
fn output_limit_counts_resolved_kinds_not_layer_switches() {
    let f = Fixture::new(
        vec![
            Spec::new(vec![SIZE], 8192, 2, 0, 1),
            Spec::new(vec![SIZE], 8192, 2, 1, 2),
        ],
        true,
    );
    let d = SparseChainDisk::with_limits(f.chain(), SparseChainDiskLimits { output_extents: 1 });
    f.reset();
    assert_eq!(d.extents(0, SIZE).unwrap().len(), 1);
    assert_eq!(f.reads(), 0);
    let f = Fixture::new(vec![Spec::new(vec![SIZE], 8192, 2, 0, 1)], true);
    let d = SparseChainDisk::with_limits(f.chain(), SparseChainDiskLimits { output_extents: 31 });
    assert!(d.extents(0, SIZE).is_err());
    assert_eq!(d.extents(0, 8192).unwrap().len(), 1);
    assert!(d.read_at(0, &mut [0; 9]).is_ok());
    let d = SparseChainDisk::with_limits(f.chain(), SparseChainDiskLimits { output_extents: 32 });
    assert_eq!(d.extents(0, SIZE).unwrap().len(), 32);
    let d = SparseChainDisk::with_limits(f.chain(), SparseChainDiskLimits { output_extents: 0 });
    assert!(d.extents(0, 0).unwrap().is_empty());
    assert!(d.extents(0, 1).is_err());
}
#[test]
fn all_descriptors_and_backings_reject_destination_aliases() {
    let f = mixed();
    let d = f.disk();
    for layer in &f.layers {
        for source in std::iter::once(&layer.descriptor).chain(&layer.backings) {
            assert!(matches!(
                d.validate_destination_identity(source.copy_endpoint().unwrap()),
                Err(Error::AliasedEndpoints)
            ));
        }
    }
    let dst = MemoryBlockDevice::new(SIZE as usize).unwrap();
    d.validate_destination_identity(dst.copy_endpoint().unwrap())
        .unwrap();
    let mut endpoint = dst.copy_endpoint().unwrap();
    endpoint.identity = None;
    assert!(d.validate_destination_identity(endpoint).is_err());
}
#[test]
fn endpoint_changes_reject_copy_preflight() {
    for mode in [8, 16] {
        let f = mixed();
        let d = f.disk();
        f.layers[2].backings[0].mode.store(mode, Ordering::Relaxed);
        assert!(d.copy_endpoint().is_err());
        assert!(
            d.validate_destination_identity(
                MemoryBlockDevice::new(1).unwrap().copy_endpoint().unwrap()
            )
            .is_err()
        );
    }
    let f = mixed();
    let d = f.disk();
    f.layers[1].descriptor.mode.store(16, Ordering::Relaxed);
    assert!(d.revalidate().is_err());
}
#[test]
fn short_reads_retry_and_parent_errors_propagate() {
    for mode in [1, 2, 4] {
        let f = Fixture::new(
            vec![
                Spec::new(vec![SIZE], 8192, 0, 0, 1),
                Spec::new(vec![SIZE], 65536, 1, 0, 33),
            ],
            true,
        );
        let d = f.disk();
        f.layers[1].backings[0].mode.store(mode, Ordering::Relaxed);
        let mut b = [0; 97];
        let result = d.read_exact_at(3, &mut b);
        if mode == 4 {
            result.unwrap();
            assert_eq!(b.as_slice(), f.oracle(3, 97));
        } else {
            assert!(result.is_err());
        }
    }
}
#[test]
fn readonly_capabilities_and_retained_handles() {
    let f = mixed();
    let d = f.disk();
    let expected = f.oracle(0, 97);
    drop(f);
    d.revalidate().unwrap();
    let mut b = [0; 97];
    d.read_exact_at(0, &mut b).unwrap();
    assert_eq!(b.as_slice(), expected);
    assert_eq!(
        d.capabilities(),
        Capabilities::READ | Capabilities::SPARSE | Capabilities::EXTENTS
    );
    assert!(d.copy_endpoint().unwrap().identity.is_none());
    assert!(d.write_at(0, &[1]).is_err());
    assert!(d.write_zero_at(0, 1).is_err());
    assert!(d.discard(0, 1).is_err());
    assert!(d.flush().is_err());
}
#[test]
fn threaded_copy_and_verification_match_resolved_oracle() {
    let f = mixed();
    let d = f.disk();
    let dst = RawDisk::new(MemoryBlockDevice::new(SIZE as usize).unwrap());
    let mover = DataMover::new(CopyOptions::with_concurrency(65536, 4096, 4).unwrap());
    mover.copy(&d, &dst).unwrap();
    Verifier::new(SIZE, 65537, 131074)
        .unwrap()
        .verify(&d, &dst)
        .unwrap();
    let mut b = vec![0; SIZE as usize];
    dst.read_exact_at(0, &mut b).unwrap();
    assert_eq!(b, f.oracle(0, b.len()));
}

#[test]
fn read_error_can_leave_a_completed_prefix_without_fallback_to_older_data() {
    let f = Fixture::new(
        vec![
            Spec::new(vec![SIZE], 8192, 2, 0, 11),
            Spec::new(vec![SIZE], 65536, 1, 0, 71),
        ],
        true,
    );
    let d = f.disk();
    f.layers[1].backings[0].mode.store(2, Ordering::Relaxed);
    let mut b = [0xcc; 16384];
    assert!(d.read_at(0, &mut b).is_err());
    assert_eq!(&b[..8192], f.oracle(0, 8192));
    assert_eq!(&b[8192..], &[0xcc; 8192]);
}
