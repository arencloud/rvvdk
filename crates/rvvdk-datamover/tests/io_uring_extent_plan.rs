#![cfg(target_os = "linux")]

use rvvdk_core::{Extent, ExtentKind};

use rvvdk_datamover::io_uring::NativeExtentPlan;

#[test]
fn accepts_dense_extent_map() {
    let extents = vec![Extent::new(0, 1024 * 1024, ExtentKind::Data).unwrap()];

    let plan = NativeExtentPlan::new(extents, 1024 * 1024).unwrap();

    assert_eq!(plan.len(), 1,);

    assert!(plan.is_data_only());

    assert_eq!(plan.disk_size(), 1024 * 1024,);

    assert_eq!(plan.extents()[0].kind(), ExtentKind::Data,);
}

#[test]
fn accepts_sparse_extent_map() {
    let extents = vec![
        Extent::new(0, 1024, ExtentKind::Data).unwrap(),
        Extent::new(1024, 2048, ExtentKind::Zero).unwrap(),
        Extent::new(3072, 4096, ExtentKind::Hole).unwrap(),
    ];

    let plan = NativeExtentPlan::new(extents, 7168).unwrap();

    assert_eq!(plan.len(), 3,);

    assert!(!plan.is_data_only());

    assert_eq!(plan.extents()[0].kind(), ExtentKind::Data,);

    assert_eq!(plan.extents()[1].kind(), ExtentKind::Zero,);

    assert_eq!(plan.extents()[2].kind(), ExtentKind::Hole,);
}

#[test]
fn accepts_empty_zero_sized_disk() {
    let plan = NativeExtentPlan::new(Vec::new(), 0).unwrap();

    assert!(plan.is_empty());

    assert_eq!(plan.disk_size(), 0,);
}

#[test]
fn rejects_gap_in_extent_map() {
    let extents = vec![
        Extent::new(0, 1024, ExtentKind::Data).unwrap(),
        Extent::new(2048, 1024, ExtentKind::Data).unwrap(),
    ];

    assert!(NativeExtentPlan::new(extents, 3072,).is_err());
}

#[test]
fn rejects_incomplete_extent_map() {
    let extents = vec![Extent::new(0, 1024, ExtentKind::Data).unwrap()];

    assert!(NativeExtentPlan::new(extents, 2048,).is_err());
}
