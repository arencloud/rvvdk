use rvvdk_core::BufferGuard;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoUringOperationKind {
    Read,
    Write,
}

pub(crate) struct InFlightOperation {
    user_data: u64,
    kind: IoUringOperationKind,
    offset: u64,
    length: usize,
    buffer: BufferGuard,
}

impl InFlightOperation {
    pub(crate) fn new(
        user_data: u64,
        kind: IoUringOperationKind,
        offset: u64,
        length: usize,
        buffer: BufferGuard,
    ) -> Self {
        Self {
            user_data,
            kind,
            offset,
            length,
            buffer,
        }
    }

    pub(crate) fn complete(self, bytes_transferred: usize) -> CompletedOperation {
        CompletedOperation {
            user_data: self.user_data,
            kind: self.kind,
            offset: self.offset,
            length: self.length,
            bytes_transferred,
            buffer: self.buffer,
        }
    }
}

pub struct CompletedOperation {
    user_data: u64,
    kind: IoUringOperationKind,
    offset: u64,
    length: usize,
    bytes_transferred: usize,
    buffer: BufferGuard,
}

impl CompletedOperation {
    pub const fn user_data(&self) -> u64 {
        self.user_data
    }

    pub const fn kind(&self) -> IoUringOperationKind {
        self.kind
    }

    pub const fn offset(&self) -> u64 {
        self.offset
    }

    pub const fn length(&self) -> usize {
        self.length
    }

    pub const fn bytes_transferred(&self) -> usize {
        self.bytes_transferred
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer.as_slice()[..self.length]
    }

    pub fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer.as_mut_slice()[..self.length]
    }

    pub(crate) fn into_buffer(self) -> BufferGuard {
        self.buffer
    }
}
