use crate::{DiskRange, Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtentKind {
    Data,
    Zero,
    Hole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Extent {
    range: DiskRange,
    kind: ExtentKind,
}

impl Extent {
    pub fn new(offset: u64, length: u64, kind: ExtentKind) -> Result<Self> {
        if length == 0 {
            return Err(Error::CorruptMetadata(
                "extent length cannot be zero".into(),
            ));
        }

        Ok(Self {
            range: DiskRange::new(offset, length)?,
            kind,
        })
    }

    pub const fn range(&self) -> DiskRange {
        self.range
    }

    pub const fn offset(&self) -> u64 {
        self.range.offset()
    }

    pub const fn length(&self) -> u64 {
        self.range.length()
    }

    pub fn end(&self) -> u64 {
        self.range.end()
    }

    pub const fn kind(&self) -> ExtentKind {
        self.kind
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_data_extent() {
        let extent = Extent::new(4096, 8192, ExtentKind::Data).unwrap();

        assert_eq!(extent.offset(), 4096);
        assert_eq!(extent.length(), 8192);
        assert_eq!(extent.end(), 12288);
        assert_eq!(extent.kind(), ExtentKind::Data);
    }

    #[test]
    fn rejects_empty_extent() {
        let result = Extent::new(0, 0, ExtentKind::Data);

        assert!(matches!(result, Err(Error::CorruptMetadata(_))));
    }
}
