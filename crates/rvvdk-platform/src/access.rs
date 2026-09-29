//! Cooperative, process-local admission for regular-file aliases.
use rvvdk_core::{Error, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock, Weak};

/// Payload mode, extent inspection, or a whole-file durability barrier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileAccessKind {
    BufferedRead,
    BufferedWrite,
    DirectRead,
    DirectWrite,
    Inspect,
    Flush,
}
impl FileAccessKind {
    fn writes(self) -> bool {
        matches!(self, Self::BufferedWrite | Self::DirectWrite | Self::Flush)
    }
    fn direct(self) -> Option<bool> {
        match self {
            Self::DirectRead | Self::DirectWrite => Some(true),
            Self::BufferedRead | Self::BufferedWrite => Some(false),
            _ => None,
        }
    }
}
#[derive(Debug)]
struct Entry {
    id: u64,
    start: u64,
    end: u64,
    kind: FileAccessKind,
}
#[derive(Debug, Default)]
struct State {
    next: u64,
    active: Vec<Entry>,
}
#[derive(Debug)]
struct Coordinator {
    state: Mutex<State>,
    page_size: u64,
}
type Identity = (u64, u64);
static REGISTRY: OnceLock<Mutex<HashMap<Identity, Weak<Coordinator>>>> = OnceLock::new();

/// Shared by participating handles for the same (device, inode). Raw syscalls,
/// mmap, external processes, and separately loaded library copies do not join it.
#[derive(Debug, Clone)]
pub struct FileAccess(Arc<Coordinator>);
impl FileAccess {
    /// Register a known regular-file identity. Keep its descriptor alive while
    /// using this handle; identities are not path names or persistent tokens.
    pub fn for_identity(device: u64, inode: u64) -> Self {
        let mut registry = REGISTRY
            .get_or_init(Mutex::default)
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(coordinator) = registry.get(&(device, inode)).and_then(Weak::upgrade) {
            return Self(coordinator);
        }
        // Clean weak entries on insertion, rather than retaining every file ever opened.
        registry.retain(|_, value| value.strong_count() != 0);
        static PAGE_SIZE: OnceLock<u64> = OnceLock::new();
        let page_size = *PAGE_SIZE.get_or_init(|| {
            // SAFETY: sysconf takes a constant selector, with no pointers.
            let size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
            u64::try_from(size)
                .ok()
                .filter(|s| *s > 0)
                .expect("Linux page size must be positive")
        });
        let coordinator = Arc::new(Coordinator {
            state: Mutex::new(State::default()),
            page_size,
        });
        registry.insert((device, inode), Arc::downgrade(&coordinator));
        Self(coordinator)
    }

    /// Admit without waiting for active I/O. A conflict submits no new request.
    /// Same-mode reads may overlap; any overlapping writer conflicts. Mixed
    /// payload modes conflict on page overlap, including adjacent byte ranges.
    /// Flush excludes all active operations for the whole file.
    pub fn try_acquire(
        &self,
        offset: u64,
        length: u64,
        kind: FileAccessKind,
    ) -> Result<Option<FileAccessGuard>> {
        let end = offset
            .checked_add(length)
            .ok_or(Error::RangeOverflow { offset, length })?;
        if length == 0 && kind != FileAccessKind::Flush {
            return Ok(None);
        }
        let (start, end) = if kind == FileAccessKind::Flush {
            (0, u64::MAX)
        } else {
            (offset, end)
        };
        let mut state = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        let conflict = state.active.iter().any(|entry| {
            if kind == FileAccessKind::Flush || entry.kind == FileAccessKind::Flush {
                return true;
            }
            let overlap = start < entry.end && entry.start < end;
            let mixed = matches!((kind.direct(),entry.kind.direct()),(Some(a),Some(b)) if a!=b);
            let page = self.0.page_size;
            (overlap && (kind.writes() || entry.kind.writes()))
                || (mixed
                    && start / page <= (entry.end - 1) / page
                    && entry.start / page <= (end - 1) / page)
        });
        if conflict {
            return Err(Error::ConcurrentFileAccess { offset, length });
        }
        state
            .active
            .try_reserve(1)
            .map_err(|_| Error::Io(std::io::ErrorKind::OutOfMemory.into()))?;
        let id = loop {
            state.next = state.next.wrapping_add(1);
            if !state.active.iter().any(|e| e.id == state.next) {
                break state.next;
            }
        };
        state.active.push(Entry {
            id,
            start,
            end,
            kind,
        });
        Ok(Some(FileAccessGuard {
            coordinator: Arc::clone(&self.0),
            id,
        }))
    }
}

/// Owned request admission. Native operations retain this through CQE confirmation;
/// unconfirmed operations must retain it along with their buffers and descriptors.
#[derive(Debug)]
pub struct FileAccessGuard {
    coordinator: Arc<Coordinator>,
    id: u64,
}
impl Drop for FileAccessGuard {
    fn drop(&mut self) {
        let mut state = self
            .coordinator
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let index = state
            .active
            .iter()
            .position(|e| e.id == self.id)
            .expect("live admission entry");
        state.active.swap_remove(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use FileAccessKind::*;

    fn access() -> FileAccess {
        FileAccess(Arc::new(Coordinator {
            state: Mutex::new(State::default()),
            page_size: 4096,
        }))
    }

    #[test]
    fn overlap_matrix_is_symmetric_and_flush_is_exclusive() {
        let kinds = [
            BufferedRead,
            BufferedWrite,
            DirectRead,
            DirectWrite,
            Inspect,
            Flush,
        ];
        for a in kinds {
            for b in kinds {
                let file = access();
                let _first = file.try_acquire(512, 512, a).unwrap();
                let allowed = matches!(
                    (a, b),
                    (BufferedRead, BufferedRead | Inspect)
                        | (DirectRead, DirectRead | Inspect)
                        | (Inspect, BufferedRead | DirectRead | Inspect)
                );
                assert_eq!(
                    file.try_acquire(512, 512, b).is_ok(),
                    allowed,
                    "{a:?}, {b:?}"
                );
                if a == Flush || b == Flush {
                    assert!(file.try_acquire(8192, 1, b).is_err());
                }
            }
        }
    }

    #[test]
    fn mixed_modes_conflict_within_page_but_same_mode_subpage_writes_do_not() {
        let file = access();
        let _first = file.try_acquire(0, 512, DirectWrite).unwrap();
        let _adjacent = file.try_acquire(512, 512, DirectWrite).unwrap();
        assert!(matches!(
            file.try_acquire(4095, 1, BufferedRead),
            Err(Error::ConcurrentFileAccess { .. })
        ));
        assert!(file.try_acquire(4096, 1, BufferedWrite).is_ok());
        assert!(file.try_acquire(1024, 1, Inspect).is_ok());
        // A mixed request ending at the page boundary touches the first page.
        assert!(file.try_acquire(4095, 2, BufferedWrite).is_err());
    }

    #[test]
    fn empty_overflow_release_and_unwind_preserve_admission() {
        let file = access();
        let barrier = file.try_acquire(0, 0, Flush).unwrap();
        assert!(file.try_acquire(u64::MAX, 0, DirectRead).unwrap().is_none());
        assert!(matches!(
            file.try_acquire(u64::MAX, 1, DirectRead),
            Err(Error::RangeOverflow { .. })
        ));
        assert!(file.try_acquire(u64::MAX - 1, 1, Inspect).is_err());
        drop(barrier);
        let result = std::panic::catch_unwind(|| {
            let _guard = file.try_acquire(0, 1, BufferedWrite).unwrap();
            panic!("release while unwinding");
        });
        assert!(result.is_err());
        assert!(file.try_acquire(0, 1, DirectWrite).is_ok());
    }

    #[test]
    fn concurrent_admission_excludes_a_writer_until_cross_thread_release() {
        let file = access();
        let guard = file.try_acquire(0, 4096, DirectRead).unwrap();
        std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    assert!(file.try_acquire(0, 1, BufferedWrite).is_err());
                    drop(guard);
                    assert!(file.try_acquire(0, 1, BufferedWrite).is_ok());
                })
                .join()
                .unwrap();
        });
    }

    #[test]
    fn wrapped_ids_skip_live_entries() {
        let file = access();
        let first = file.try_acquire(0, 1, BufferedRead).unwrap();
        file.0.state.lock().unwrap().next = u64::MAX;
        let second = file.try_acquire(0, 1, BufferedRead).unwrap();
        assert_ne!(first.as_ref().unwrap().id, second.as_ref().unwrap().id);
        drop(first);
        assert!(file.try_acquire(0, 1, BufferedWrite).is_err());
        drop(second);
        assert!(file.try_acquire(0, 1, BufferedWrite).is_ok());
    }
}
