use rvvdk_core::{Error, Extent, ExtentKind, Result};

use crate::work::{WorkItem, WorkKind};

pub(crate) fn plan_extent(extent: Extent, block_size: usize) -> Result<Vec<WorkItem>> {
    let kind = match extent.kind() {
        ExtentKind::Data => WorkKind::Copy,
        ExtentKind::Zero => WorkKind::Zero,
        ExtentKind::Hole => WorkKind::Discard,
    };

    let mut items = Vec::new();

    let mut offset = extent.offset();
    let end = extent.end();

    while offset < end {
        let remaining = end - offset;

        let length_u64 = remaining.min(block_size as u64);

        let length = usize::try_from(length_u64).map_err(|_| Error::RangeOverflow {
            offset,
            length: length_u64,
        })?;

        items.push(WorkItem::new(offset, length, kind));

        offset = offset.checked_add(length_u64).ok_or(Error::RangeOverflow {
            offset,
            length: length_u64,
        })?;
    }

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_extent_into_blocks() {
        let extent = Extent::new(1000, 2500, ExtentKind::Data).unwrap();

        let items = plan_extent(extent, 1024).unwrap();

        assert_eq!(items.len(), 3,);

        assert_eq!(items[0].offset(), 1000,);

        assert_eq!(items[0].length(), 1024,);

        assert_eq!(items[1].offset(), 2024,);

        assert_eq!(items[1].length(), 1024,);

        assert_eq!(items[2].offset(), 3048,);

        assert_eq!(items[2].length(), 452,);
    }

    #[test]
    fn maps_data_to_copy() {
        let extent = Extent::new(0, 4096, ExtentKind::Data).unwrap();

        let items = plan_extent(extent, 4096).unwrap();

        assert_eq!(items[0].kind(), WorkKind::Copy,);
    }

    #[test]
    fn maps_zero_to_zero() {
        let extent = Extent::new(0, 4096, ExtentKind::Zero).unwrap();

        let items = plan_extent(extent, 4096).unwrap();

        assert_eq!(items[0].kind(), WorkKind::Zero,);
    }

    #[test]
    fn maps_hole_to_discard() {
        let extent = Extent::new(0, 4096, ExtentKind::Hole).unwrap();

        let items = plan_extent(extent, 4096).unwrap();

        assert_eq!(items[0].kind(), WorkKind::Discard,);
    }
}
