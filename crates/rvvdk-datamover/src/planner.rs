use rvvdk_core::{Error, Extent, ExtentKind, Result};

use crate::work::{WorkItem, WorkKind};

pub(crate) struct ExtentWorkIter {
    next_offset: u64,
    end: u64,
    block_size: usize,
    kind: WorkKind,
}

impl ExtentWorkIter {
    pub(crate) fn new(extent: Extent, block_size: usize) -> Self {
        let kind = match extent.kind() {
            ExtentKind::Data => WorkKind::Copy,

            ExtentKind::Zero => WorkKind::Zero,

            ExtentKind::Hole => WorkKind::Discard,
        };

        Self {
            next_offset: extent.offset(),
            end: extent.end(),
            block_size,
            kind,
        }
    }
}

impl Iterator for ExtentWorkIter {
    type Item = Result<WorkItem>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_offset >= self.end {
            return None;
        }

        let offset = self.next_offset;

        let remaining = self.end - offset;

        let length_u64 = remaining.min(self.block_size as u64);

        let length = match usize::try_from(length_u64) {
            Ok(length) => length,

            Err(_) => {
                return Some(Err(Error::RangeOverflow {
                    offset,
                    length: length_u64,
                }));
            }
        };

        self.next_offset = match offset.checked_add(length_u64) {
            Some(next) => next,

            None => {
                return Some(Err(Error::RangeOverflow {
                    offset,
                    length: length_u64,
                }));
            }
        };

        Some(Ok(WorkItem::new(offset, length, self.kind)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn iterator_streams_extent_blocks() {
        let extent = Extent::new(1000, 2500, ExtentKind::Data).unwrap();

        let mut iterator = ExtentWorkIter::new(extent, 1024);

        let first = iterator.next().unwrap().unwrap();

        assert_eq!(first.offset(), 1000,);

        assert_eq!(first.length(), 1024,);

        let second = iterator.next().unwrap().unwrap();

        assert_eq!(second.offset(), 2024,);

        let third = iterator.next().unwrap().unwrap();

        assert_eq!(third.offset(), 3048,);

        assert_eq!(third.length(), 452,);

        assert!(iterator.next().is_none());
    }

    #[test]
    fn large_extent_is_lazy() {
        let extent = Extent::new(0, 10 * 1024 * 1024 * 1024, ExtentKind::Data).unwrap();

        let mut iterator = ExtentWorkIter::new(extent, 1024 * 1024);

        for expected in 0..10 {
            let work = iterator.next().unwrap().unwrap();

            assert_eq!(work.offset(), expected * 1024 * 1024,);
        }
    }
}
