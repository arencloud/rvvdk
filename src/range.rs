use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiskRange {
    offset: u64,
    length: u64,
}

impl DiskRange {
    pub fn new(offset: u64, length: u64) -> Result<Self> {
        offset
            .checked_add(length)
            .ok_or(Error::RangeOverflow { offset, length })?;

        Ok(Self { offset, length })
    }

    pub const fn offset(&self) -> u64 {
        self.offset
    }

    pub const fn length(&self) -> u64 {
        self.length
    }

    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn end(&self) -> u64 {
        self.offset + self.length
    }

    pub fn contains(&self, offset: u64) -> bool {
        offset >= self.offset && offset < self.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_valid_range() {
        let range = DiskRange::new(1024, 4096).unwrap();

        assert_eq!(range.offset(), 1024);
        assert_eq!(range.length(), 4096);
        assert_eq!(range.end(), 5120);
    }

    #[test]
    fn empty_range_is_valid() {
        let range = DiskRange::new(4096, 0).unwrap();

        assert!(range.is_empty());
        assert_eq!(range.end(), 4096);
    }

    #[test]
    fn detects_overflow() {
        let result = DiskRange::new(u64::MAX, 1);

        assert!(matches!(result, Err(Error::RangeOverflow { .. })));
    }

    #[test]
    fn contains_offset_inside_range() {
        let range = DiskRange::new(100, 100).unwrap();

        assert!(range.contains(100));
        assert!(range.contains(150));
        assert!(range.contains(199));

        assert!(!range.contains(99));
        assert!(!range.contains(200));
    }
}
