use std::alloc::{Layout, alloc_zeroed, dealloc};
use std::ptr::NonNull;

use crate::{Error, Result};

pub struct AlignedBuffer {
    ptr: NonNull<u8>,
    len: usize,
    alignment: usize,
}

// SAFETY:
//
// `AlignedBuffer` exclusively owns the allocation referenced by `ptr`.
//
// Moving the buffer to another thread transfers ownership of that
// allocation. The allocation comes from the global allocator and does
// not reference thread-local state.
//
// Mutable access requires `&mut self`, so moving ownership between
// threads does not introduce concurrent mutable access.
unsafe impl Send for AlignedBuffer {}

impl AlignedBuffer {
    pub fn new(len: usize, alignment: usize) -> Result<Self> {
        if alignment == 0 || !alignment.is_power_of_two() {
            return Err(Error::InvalidBufferAlignment { alignment });
        }

        if len == 0 {
            return Err(Error::BufferAllocation {
                size: len,
                alignment,
            });
        }

        let layout =
            Layout::from_size_align(len, alignment).map_err(|_| Error::BufferAllocation {
                size: len,
                alignment,
            })?;

        // SAFETY:
        // `layout` has been validated by
        // `Layout::from_size_align`.
        //
        // The returned allocation is checked for null
        // before constructing `NonNull`.
        let ptr = unsafe { alloc_zeroed(layout) };

        let ptr = NonNull::new(ptr).ok_or(Error::BufferAllocation {
            size: len,
            alignment,
        })?;

        Ok(Self {
            ptr,
            len,
            alignment,
        })
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn alignment(&self) -> usize {
        self.alignment
    }

    pub fn as_slice(&self) -> &[u8] {
        // SAFETY:
        // `ptr` references an allocation of exactly
        // `len` bytes owned by this object.
        //
        // The allocation remains valid for the lifetime
        // of `self`.
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY:
        // `ptr` references an allocation of exactly
        // `len` bytes owned exclusively by this object.
        //
        // `&mut self` guarantees exclusive access.
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    pub fn fill(&mut self, value: u8) {
        self.as_mut_slice().fill(value);
    }

    pub fn address(&self) -> usize {
        self.ptr.as_ptr() as usize
    }
}

impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.len, self.alignment)
            .expect("AlignedBuffer stored an invalid layout");

        // SAFETY:
        // `ptr` was allocated with this exact layout in
        // `AlignedBuffer::new` and has not been freed.
        unsafe {
            dealloc(self.ptr.as_ptr(), layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_aligned_buffer() {
        let buffer = AlignedBuffer::new(1024 * 1024, 4096).unwrap();

        assert_eq!(buffer.len(), 1024 * 1024,);

        assert_eq!(buffer.alignment(), 4096,);

        assert_eq!(buffer.address() % 4096, 0,);
    }

    #[test]
    fn buffer_is_zero_initialized() {
        let buffer = AlignedBuffer::new(4096, 4096).unwrap();

        assert!(buffer.as_slice().iter().all(|value| *value == 0));
    }

    #[test]
    fn buffer_can_be_modified() {
        let mut buffer = AlignedBuffer::new(4096, 4096).unwrap();

        buffer.as_mut_slice()[0..5].copy_from_slice(b"rvvdk");

        assert_eq!(&buffer.as_slice()[0..5], b"rvvdk",);
    }

    #[test]
    fn fill_changes_complete_buffer() {
        let mut buffer = AlignedBuffer::new(4096, 4096).unwrap();

        buffer.fill(0xaa);

        assert!(buffer.as_slice().iter().all(|value| *value == 0xaa));
    }

    #[test]
    fn rejects_zero_alignment() {
        let result = AlignedBuffer::new(4096, 0);

        assert!(matches!(result, Err(Error::InvalidBufferAlignment { .. })));
    }

    #[test]
    fn rejects_non_power_of_two_alignment() {
        let result = AlignedBuffer::new(4096, 1000);

        assert!(matches!(result, Err(Error::InvalidBufferAlignment { .. })));
    }
}
