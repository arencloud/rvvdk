use std::sync::{Arc, Condvar, Mutex};

use crate::{AlignedBuffer, Error, Result};

struct BufferPoolInner {
    buffers: Mutex<Vec<AlignedBuffer>>,
    available: Condvar,
    capacity: usize,
    buffer_size: usize,
    alignment: usize,
}

#[derive(Clone)]
pub struct BufferPool {
    inner: Arc<BufferPoolInner>,
}

impl BufferPool {
    pub fn new(capacity: usize, buffer_size: usize, alignment: usize) -> Result<Self> {
        if capacity == 0 {
            return Err(Error::InvalidBufferPoolCapacity);
        }

        let mut buffers = Vec::with_capacity(capacity);

        for _ in 0..capacity {
            buffers.push(AlignedBuffer::new(buffer_size, alignment)?);
        }

        Ok(Self {
            inner: Arc::new(BufferPoolInner {
                buffers: Mutex::new(buffers),
                available: Condvar::new(),
                capacity,
                buffer_size,
                alignment,
            }),
        })
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity
    }

    pub fn buffer_size(&self) -> usize {
        self.inner.buffer_size
    }

    pub fn alignment(&self) -> usize {
        self.inner.alignment
    }

    pub fn acquire(&self) -> BufferGuard {
        let mut buffers = self
            .inner
            .buffers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        loop {
            if let Some(buffer) = buffers.pop() {
                return BufferGuard {
                    buffer: Some(buffer),
                    pool: self.clone(),
                };
            }

            buffers = self
                .inner
                .available
                .wait(buffers)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    fn release(&self, buffer: AlignedBuffer) {
        let mut buffers = self
            .inner
            .buffers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        buffers.push(buffer);

        self.inner.available.notify_one();
    }

    pub fn available(&self) -> usize {
        self.inner
            .buffers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }
}

pub struct BufferGuard {
    buffer: Option<AlignedBuffer>,
    pool: BufferPool,
}

impl BufferGuard {
    pub fn buffer(&self) -> &AlignedBuffer {
        self.buffer
            .as_ref()
            .expect("BufferGuard must contain a buffer")
    }

    pub fn buffer_mut(&mut self) -> &mut AlignedBuffer {
        self.buffer
            .as_mut()
            .expect("BufferGuard must contain a buffer")
    }

    pub fn as_slice(&self) -> &[u8] {
        self.buffer().as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.buffer_mut().as_mut_slice()
    }
}

impl Drop for BufferGuard {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            self.pool.release(buffer);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn creates_pool() {
        let pool = BufferPool::new(4, 1024 * 1024, 4096).unwrap();

        assert_eq!(pool.capacity(), 4,);

        assert_eq!(pool.buffer_size(), 1024 * 1024,);

        assert_eq!(pool.alignment(), 4096,);

        assert_eq!(pool.available(), 4,);
    }

    #[test]
    fn rejects_zero_capacity() {
        let result = BufferPool::new(0, 4096, 4096);

        assert!(matches!(result, Err(Error::InvalidBufferPoolCapacity)));
    }

    #[test]
    fn acquire_removes_buffer() {
        let pool = BufferPool::new(2, 4096, 4096).unwrap();

        let _guard = pool.acquire();

        assert_eq!(pool.available(), 1,);
    }

    #[test]
    fn drop_returns_buffer() {
        let pool = BufferPool::new(2, 4096, 4096).unwrap();

        {
            let _guard = pool.acquire();

            assert_eq!(pool.available(), 1,);
        }

        assert_eq!(pool.available(), 2,);
    }

    #[test]
    fn acquired_buffer_is_aligned() {
        let pool = BufferPool::new(1, 4096, 4096).unwrap();

        let guard = pool.acquire();

        assert_eq!(guard.buffer().address() % 4096, 0,);
    }

    #[test]
    fn buffer_is_reused_after_release() {
        let pool = BufferPool::new(1, 4096, 4096).unwrap();

        let first_address = {
            let guard = pool.acquire();

            guard.buffer().address()
        };

        let second_address = {
            let guard = pool.acquire();

            guard.buffer().address()
        };

        assert_eq!(first_address, second_address,);
    }

    #[test]
    fn buffer_pool_is_send_and_sync() {
        fn assert_send_sync<T>()
        where
            T: Send + Sync,
        {
        }

        assert_send_sync::<BufferPool>();
    }

    #[test]
    fn acquire_waits_until_buffer_is_returned() {
        let pool = BufferPool::new(1, 4096, 4096).unwrap();

        let first = pool.acquire();

        let worker_pool = pool.clone();

        let (sender, receiver) = mpsc::channel();

        let worker = thread::spawn(move || {
            let _second = worker_pool.acquire();

            sender.send(()).unwrap();
        });

        assert!(receiver.recv_timeout(Duration::from_millis(50,),).is_err());

        drop(first);

        receiver.recv_timeout(Duration::from_secs(1)).unwrap();

        worker.join().unwrap();
    }
}
