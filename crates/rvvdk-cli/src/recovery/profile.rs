//! Test-only disjoint adapter spans; absent from production builds.
use serde_json::{Value, json};
use std::{cell::RefCell, collections::BTreeMap, time::Instant};
struct State {
    last: Instant,
    cpu: f64,
    phases: BTreeMap<&'static str, Value>,
}
thread_local! {static STATE:RefCell<Option<State>>=const{RefCell::new(None)};}
pub(super) fn usage() -> (f64, i64) {
    let mut r = std::mem::MaybeUninit::<libc::rusage>::uninit();
    // SAFETY: valid output pointer, only read after success.
    assert_eq!(
        unsafe { libc::getrusage(libc::RUSAGE_SELF, r.as_mut_ptr()) },
        0
    );
    // SAFETY: successful getrusage initialized the output.
    let r = unsafe { r.assume_init() };
    (
        r.ru_utime.tv_sec as f64 * 1000.
            + r.ru_utime.tv_usec as f64 / 1000.
            + r.ru_stime.tv_sec as f64 * 1000.
            + r.ru_stime.tv_usec as f64 / 1000.,
        r.ru_maxrss,
    )
}
pub(super) fn start(enabled: bool) {
    STATE.with_borrow_mut(|v| {
        assert!(v.is_none());
        if enabled {
            *v = Some(State {
                last: Instant::now(),
                cpu: usage().0,
                phases: BTreeMap::new(),
            });
        }
    });
}
pub(crate) fn mark(name: &'static str) {
    STATE.with_borrow_mut(|s|{if let Some(s)=s{let now=Instant::now();let cpu=usage().0;assert!(s.phases.insert(name,json!({"wall_ms":now.duration_since(s.last).as_secs_f64()*1000.,"cpu_ms":cpu-s.cpu})).is_none());s.last=now;s.cpu=cpu;}});
}
pub(super) fn finish() -> BTreeMap<&'static str, Value> {
    STATE.with_borrow_mut(|s| s.take().map(|s| s.phases).unwrap_or_default())
}
