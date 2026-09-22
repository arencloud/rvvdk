use rvvdk_core::{Error, Extent, ExtentKind, Result};

use crate::ExecutionBackend;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CopyPlanSummary {
    logical_bytes: u64,
    data_bytes: u64,
    zero_bytes: u64,
    hole_bytes: u64,
}

impl CopyPlanSummary {
    fn from_extents(logical_bytes: u64, extents: &[Extent]) -> Result<Self> {
        let mut data_bytes = 0_u64;

        let mut zero_bytes = 0_u64;

        let mut hole_bytes = 0_u64;

        for extent in extents {
            match extent.kind() {
                ExtentKind::Data => {
                    data_bytes =
                        data_bytes
                            .checked_add(extent.length())
                            .ok_or(Error::RangeOverflow {
                                offset: data_bytes,
                                length: extent.length(),
                            })?;
                }

                ExtentKind::Zero => {
                    zero_bytes =
                        zero_bytes
                            .checked_add(extent.length())
                            .ok_or(Error::RangeOverflow {
                                offset: zero_bytes,
                                length: extent.length(),
                            })?;
                }

                ExtentKind::Hole => {
                    hole_bytes =
                        hole_bytes
                            .checked_add(extent.length())
                            .ok_or(Error::RangeOverflow {
                                offset: hole_bytes,
                                length: extent.length(),
                            })?;
                }
            }
        }

        let data_and_zero = data_bytes
            .checked_add(zero_bytes)
            .ok_or(Error::RangeOverflow {
                offset: data_bytes,
                length: zero_bytes,
            })?;

        let accounted = data_and_zero
            .checked_add(hole_bytes)
            .ok_or(Error::RangeOverflow {
                offset: data_and_zero,
                length: hole_bytes,
            })?;

        if accounted != logical_bytes {
            return Err(Error::CorruptMetadata(format!(
                "extent bytes do not match logical size: \
                         logical={logical_bytes}, \
                         accounted={accounted}"
            )));
        }

        Ok(Self {
            logical_bytes,
            data_bytes,
            zero_bytes,
            hole_bytes,
        })
    }

    pub const fn logical_bytes(&self) -> u64 {
        self.logical_bytes
    }

    pub const fn data_bytes(&self) -> u64 {
        self.data_bytes
    }

    pub const fn zero_bytes(&self) -> u64 {
        self.zero_bytes
    }

    pub const fn hole_bytes(&self) -> u64 {
        self.hole_bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyPlan {
    summary: CopyPlanSummary,
    extents: Vec<Extent>,
    extent_fingerprint: u64,
    backend: ExecutionBackend,
    block_size: usize,
    alignment: usize,
}

impl CopyPlan {
    pub(crate) fn new(
        logical_bytes: u64,
        extents: Vec<Extent>,
        backend: ExecutionBackend,
        block_size: usize,
        alignment: usize,
    ) -> Result<Self> {
        let summary = CopyPlanSummary::from_extents(logical_bytes, &extents)?;
        let extent_fingerprint = extent_fingerprint(&extents);

        Ok(Self {
            summary,
            extent_fingerprint,
            extents,
            backend,
            block_size,
            alignment,
        })
    }

    pub const fn extent_fingerprint(&self) -> u64 {
        self.extent_fingerprint
    }

    pub const fn summary(&self) -> &CopyPlanSummary {
        &self.summary
    }

    pub const fn logical_bytes(&self) -> u64 {
        self.summary.logical_bytes()
    }

    pub const fn data_bytes(&self) -> u64 {
        self.summary.data_bytes()
    }

    pub const fn zero_bytes(&self) -> u64 {
        self.summary.zero_bytes()
    }

    pub const fn hole_bytes(&self) -> u64 {
        self.summary.hole_bytes()
    }

    pub const fn extent_count(&self) -> usize {
        self.extents.len()
    }

    pub fn extents(&self) -> &[Extent] {
        &self.extents
    }

    pub const fn backend(&self) -> ExecutionBackend {
        self.backend
    }

    pub const fn block_size(&self) -> usize {
        self.block_size
    }

    pub const fn alignment(&self) -> usize {
        self.alignment
    }
}

pub(crate) fn extent_fingerprint(extents: &[Extent]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

    const PRIME: u64 = 0x0000_0100_0000_01b3;

    fn mix(mut hash: u64, value: u64) -> u64 {
        for byte in value.to_le_bytes() {
            hash ^= u64::from(byte);

            hash = hash.wrapping_mul(PRIME);
        }

        hash
    }

    let mut hash = OFFSET_BASIS;

    for extent in extents {
        hash = mix(hash, extent.offset());

        hash = mix(hash, extent.length());

        let kind = match extent.kind() {
            ExtentKind::Data => 1_u64,

            ExtentKind::Zero => 2_u64,

            ExtentKind::Hole => 3_u64,
        };

        hash = mix(hash, kind);
    }

    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIB: u64 = 1024 * 1024;

    fn mixed_extents() -> Vec<Extent> {
        vec![
            Extent::new(0, 2 * MIB, ExtentKind::Data).unwrap(),
            Extent::new(2 * MIB, MIB, ExtentKind::Zero).unwrap(),
            Extent::new(3 * MIB, MIB, ExtentKind::Hole).unwrap(),
        ]
    }

    #[test]
    fn summarizes_extents() {
        let extents = mixed_extents();

        let summary = CopyPlanSummary::from_extents(4 * MIB, &extents).unwrap();

        assert_eq!(summary.logical_bytes(), 4 * MIB,);

        assert_eq!(summary.data_bytes(), 2 * MIB,);

        assert_eq!(summary.zero_bytes(), MIB,);

        assert_eq!(summary.hole_bytes(), MIB,);
    }

    #[test]
    fn stores_copy_plan_metadata() {
        let extents = mixed_extents();

        let plan = CopyPlan::new(
            4 * MIB,
            extents.clone(),
            ExecutionBackend::IoUring,
            64 * 1024,
            4096,
        )
        .unwrap();

        assert_eq!(plan.logical_bytes(), 4 * MIB,);

        assert_eq!(plan.data_bytes(), 2 * MIB,);

        assert_eq!(plan.zero_bytes(), MIB,);

        assert_eq!(plan.hole_bytes(), MIB,);

        assert_eq!(plan.extent_count(), 3,);

        assert_eq!(plan.extents(), extents.as_slice(),);

        assert_eq!(plan.backend(), ExecutionBackend::IoUring,);

        assert_eq!(plan.block_size(), 64 * 1024,);

        assert_eq!(plan.alignment(), 4096,);

        assert_eq!(plan.summary().logical_bytes(), 4 * MIB,);
    }

    #[test]
    fn supports_empty_plan() {
        let plan = CopyPlan::new(0, Vec::new(), ExecutionBackend::Threaded, 64 * 1024, 1).unwrap();

        assert_eq!(plan.logical_bytes(), 0,);

        assert_eq!(plan.data_bytes(), 0,);

        assert_eq!(plan.zero_bytes(), 0,);

        assert_eq!(plan.hole_bytes(), 0,);

        assert_eq!(plan.extent_count(), 0,);

        assert!(plan.extents().is_empty());

        assert_eq!(plan.backend(), ExecutionBackend::Threaded,);

        assert_eq!(plan.block_size(), 64 * 1024,);

        assert_eq!(plan.alignment(), 1,);
    }

    #[test]
    fn rejects_extent_bytes_that_do_not_match_logical_size() {
        let extents = vec![Extent::new(0, 2 * MIB, ExtentKind::Data).unwrap()];

        let result = CopyPlan::new(4 * MIB, extents, ExecutionBackend::Threaded, 64 * 1024, 1);

        assert!(matches!(result, Err(Error::CorruptMetadata(_))));
    }

    #[test]
    fn rejects_accounting_larger_than_logical_size() {
        let extents = vec![
            Extent::new(0, 2 * MIB, ExtentKind::Data).unwrap(),
            Extent::new(2 * MIB, 2 * MIB, ExtentKind::Zero).unwrap(),
        ];

        let result = CopyPlan::new(3 * MIB, extents, ExecutionBackend::Threaded, 64 * 1024, 1);

        assert!(matches!(result, Err(Error::CorruptMetadata(_))));
    }

    #[test]
    fn identical_extent_maps_have_identical_fingerprints() {
        let first = mixed_extents();

        let second = mixed_extents();

        assert_eq!(extent_fingerprint(&first,), extent_fingerprint(&second,),);
    }

    #[test]
    fn extent_kind_changes_fingerprint() {
        let first = vec![Extent::new(0, 4 * MIB, ExtentKind::Data).unwrap()];

        let second = vec![Extent::new(0, 4 * MIB, ExtentKind::Hole).unwrap()];

        assert_ne!(extent_fingerprint(&first,), extent_fingerprint(&second,),);
    }

    #[test]
    fn extent_boundary_changes_fingerprint() {
        let first = vec![Extent::new(0, 4 * MIB, ExtentKind::Data).unwrap()];

        let second = vec![
            Extent::new(0, 2 * MIB, ExtentKind::Data).unwrap(),
            Extent::new(2 * MIB, 2 * MIB, ExtentKind::Data).unwrap(),
        ];

        assert_ne!(extent_fingerprint(&first,), extent_fingerprint(&second,),);
    }
}
