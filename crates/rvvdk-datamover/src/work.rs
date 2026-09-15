#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkKind {
    Copy,
    Zero,
    Discard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WorkItem {
    offset: u64,
    length: usize,
    kind: WorkKind,
}

impl WorkItem {
    pub(crate) const fn new(offset: u64, length: usize, kind: WorkKind) -> Self {
        Self {
            offset,
            length,
            kind,
        }
    }

    pub(crate) const fn offset(&self) -> u64 {
        self.offset
    }

    pub(crate) const fn length(&self) -> usize {
        self.length
    }

    pub(crate) const fn kind(&self) -> WorkKind {
        self.kind
    }
}
