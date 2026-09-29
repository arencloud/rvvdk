use rvvdk_datamover::Cancellation;
use std::sync::atomic::{AtomicI32, Ordering};
static SIGNAL: AtomicI32 = AtomicI32::new(0);
extern "C" fn cancel(signal: libc::c_int) {
    // Lock-free atomic only: no allocation, locks, formatting or cleanup in handler.
    let _ = SIGNAL.compare_exchange(0, signal, Ordering::Relaxed, Ordering::Relaxed);
}
struct Signals;
impl Cancellation for Signals {
    fn is_cancelled(&self) -> bool {
        SIGNAL.load(Ordering::Relaxed) != 0
    }
}
struct Handlers(Vec<(libc::c_int, libc::sigaction)>);
impl Handlers {
    fn install() -> std::io::Result<Self> {
        let mut guard = Self(Vec::new());
        for signal in [libc::SIGINT, libc::SIGTERM] {
            // SAFETY: valid initialized sigaction structures; handler has C ABI and
            // static lifetime. Main owns installation before creating job workers.
            unsafe {
                let mut action: libc::sigaction = std::mem::zeroed();
                let mut old: libc::sigaction = std::mem::zeroed();
                action.sa_sigaction = cancel as *const () as usize;
                action.sa_flags = libc::SA_RESTART;
                libc::sigemptyset(&mut action.sa_mask);
                if libc::sigaction(signal, &action, &mut old) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                guard.0.push((signal, old));
            }
        }
        Ok(guard)
    }
}
impl Drop for Handlers {
    fn drop(&mut self) {
        for (signal, old) in self.0.iter().rev() {
            // SAFETY: restore the previously returned dispositions.
            unsafe {
                libc::sigaction(*signal, old, std::ptr::null_mut());
            }
        }
    }
}
fn main() -> std::process::ExitCode {
    let arguments: Vec<_> = std::env::args_os().collect();
    let transfer = arguments
        .iter()
        .skip(1)
        .find(|a| !a.as_encoded_bytes().starts_with(b"-"))
        .is_some_and(|a| a == "copy" || a == "verify");
    let _handlers = if transfer {
        match Handlers::install() {
            Ok(h) => Some(h),
            Err(e) => {
                eprintln!("rvddk: install cancellation handlers: {e}");
                return 1.into();
            }
        }
    } else {
        None
    };
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    let mut err = std::io::stderr().lock();
    let code = rvvdk_cli::run_with_cancellation(arguments, &mut out, &mut err, &Signals);
    // Operational failures retain exit 1 even if a cancellation signal also arrived.
    let code = if code == 130 && SIGNAL.load(Ordering::Relaxed) == libc::SIGTERM {
        143
    } else {
        code
    };
    std::process::ExitCode::from(code as u8)
}
