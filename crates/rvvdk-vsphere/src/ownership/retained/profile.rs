//! Opt-in, test-only local profiling. Never compiled into the production library.
use serde::Serialize;
use std::{cell::RefCell, collections::BTreeMap, marker::PhantomData, rc::Rc, time::Instant};

#[derive(Clone, Debug, Default, Serialize)]
pub(crate) struct Metric {
    pub calls: u64,
    pub bytes: u64,
    pub wall_ms: f64,
    pub cpu_ms: f64,
}
#[derive(Default)]
struct State {
    active: bool,
    enabled: bool,
    depth: usize,
    metrics: BTreeMap<&'static str, Metric>,
}
thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}
fn cpu_ns() -> u64 {
    let mut t = std::mem::MaybeUninit::<libc::timespec>::uninit();
    // SAFETY: writable output, read only after successful initialization.
    assert_eq!(
        unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, t.as_mut_ptr()) },
        0
    );
    // SAFETY: clock_gettime succeeded.
    let t = unsafe { t.assume_init() };
    t.tv_sec as u64 * 1_000_000_000 + t.tv_nsec as u64
}
pub(crate) struct Session(PhantomData<Rc<()>>);
impl Session {
    pub(crate) fn start(enabled: bool) -> Self {
        STATE.with_borrow_mut(|s| {
            assert!(!s.active && s.depth == 0 && s.metrics.is_empty());
            s.active = true;
            s.enabled = enabled;
        });
        Self(PhantomData)
    }
    pub(crate) fn finish(self) -> BTreeMap<&'static str, Metric> {
        STATE.with_borrow_mut(|s| {
            assert_eq!(s.depth, 0);
            std::mem::take(&mut s.metrics)
        })
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        STATE.with_borrow_mut(|s| *s = State::default());
    }
}
pub(crate) struct Scope {
    name: &'static str,
    bytes: u64,
    wall: Instant,
    cpu: u64,
    _thread: PhantomData<Rc<()>>,
}
pub(crate) fn enter(name: &'static str, bytes: u64) -> Option<Scope> {
    STATE.with_borrow_mut(|s| {
        if !s.enabled {
            return None;
        }
        // Regions are deliberately disjoint: nested spans would double count.
        assert_eq!(s.depth, 0, "profiling regions must not nest");
        s.depth = 1;
        Some(Scope {
            name,
            bytes,
            wall: Instant::now(),
            cpu: cpu_ns(),
            _thread: PhantomData,
        })
    })
}
impl Drop for Scope {
    fn drop(&mut self) {
        let cpu = cpu_ns() - self.cpu;
        let wall = self.wall.elapsed().as_secs_f64() * 1000.;
        STATE.with_borrow_mut(|s| {
            assert_eq!(s.depth, 1);
            s.depth = 0;
            let m = s.metrics.entry(self.name).or_default();
            m.calls += 1;
            m.bytes += self.bytes;
            m.wall_ms += wall;
            m.cpu_ms += cpu as f64 / 1e6;
        });
    }
}

#[test]
fn isolation_disabled_mode_and_byte_accounting() {
    let s = Session::start(false);
    assert!(enter("disabled", 7).is_none());
    assert!(s.finish().is_empty());
    let s = Session::start(true);
    for _ in 0..2 {
        drop(enter("read", 17));
    }
    std::thread::spawn(|| assert!(enter("other_thread", 1).is_none()))
        .join()
        .unwrap();
    let m = s.finish();
    assert_eq!(m.len(), 1);
    assert_eq!((m["read"].calls, m["read"].bytes), (2, 34));
    assert!(m["read"].wall_ms.is_finite() && m["read"].cpu_ms.is_finite());
}

#[test]
fn unwinding_releases_measurement_state() {
    assert!(
        std::panic::catch_unwind(|| {
            let _s = Session::start(true);
            let _span = enter("failed", 1);
            panic!("synthetic failure");
        })
        .is_err()
    );
    assert!(Session::start(true).finish().is_empty());
}
