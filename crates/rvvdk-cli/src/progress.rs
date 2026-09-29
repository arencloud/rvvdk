use crate::error::Failure;
use rvvdk_core::CopyProgress;
use rvvdk_datamover::{Cancellation, CopyEvent, CopyPhase};
use serde_json::json;
use std::{
    cell::{Cell, RefCell},
    io::Write,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

pub(crate) struct Feedback<'a, W> {
    writer: RefCell<&'a mut W>,
    enabled: bool,
    json: bool,
    pub stopped: AtomicBool,
    pub error: RefCell<Option<std::io::Error>>,
    pub copied: Cell<CopyProgress>,
    pub verified: Cell<u64>,
    started: Instant,
    sequence: Cell<u64>,
    total: Cell<u64>,
    last_verify: Cell<(u64, Duration)>,
}
impl<'a, W: Write> Feedback<'a, W> {
    pub fn new(writer: &'a mut W, enabled: bool, json: bool) -> Self {
        Self {
            writer: RefCell::new(writer),
            enabled,
            json,
            stopped: AtomicBool::new(false),
            error: RefCell::new(None),
            copied: Cell::new(CopyProgress::default()),
            verified: Cell::new(0),
            started: Instant::now(),
            sequence: Cell::new(0),
            total: Cell::new(0),
            last_verify: Cell::new((0, Duration::ZERO)),
        }
    }
    pub fn emit(
        &self,
        phase: &str,
        logical_bytes: u64,
        backend: Option<&str>,
        destination_state: Option<&str>,
    ) {
        if logical_bytes > 0 {
            self.total.set(logical_bytes);
        }
        let logical_bytes = self.total.get();
        if !self.enabled || self.stopped.load(Ordering::Relaxed) {
            return;
        }
        let sequence = self.sequence.get() + 1;
        self.sequence.set(sequence);
        let p = self.copied.get();
        let completed = p.bytes_written + p.bytes_zeroed + p.bytes_discarded;
        let mut writer = self.writer.borrow_mut();
        let result = if self.json {
            let event = json!({"schema_version":1,"event":"progress","sequence":sequence,"phase":phase,"logical_bytes":logical_bytes,"logical_bytes_processed":completed,"bytes_verified":self.verified.get(),"backend":backend,"destination_state":destination_state,"elapsed_seconds":self.started.elapsed().as_secs_f64(),"confirmed":{"bytes_read":p.bytes_read,"bytes_written":p.bytes_written,"bytes_zeroed":p.bytes_zeroed,"bytes_discarded":p.bytes_discarded,"unconfirmed_io":p.unconfirmed_io}});
            serde_json::to_writer(&mut **writer, &event)
                .map_err(std::io::Error::other)
                .and_then(|_| writeln!(writer))
        } else {
            writeln!(
                writer,
                "{phase}: {completed}/{logical_bytes} bytes processed, {} bytes verified",
                self.verified.get()
            )
        };
        if let Err(error) = result.and_then(|_| writer.flush()) {
            *self.error.borrow_mut() = Some(error);
            self.stopped.store(true, Ordering::Release);
        }
    }
    pub fn engine(&self, event: &CopyEvent) {
        self.copied.set(event.progress);
        let phase = match event.phase {
            CopyPhase::Preparing => "copy_preparing",
            CopyPhase::Started => "copy_started",
            CopyPhase::Transferring => "copying",
            CopyPhase::Flushing => "copy_flushing",
            CopyPhase::Completed => "copy_flushed",
            _ => return,
        };
        self.emit(
            phase,
            event.logical_bytes,
            event.backend.map(|b| {
                if b == rvvdk_datamover::ExecutionBackend::IoUring {
                    "io-uring"
                } else {
                    "threaded"
                }
            }),
            None,
        );
    }
    pub fn verification(&self, bytes: u64, total: u64) {
        self.verified.set(bytes);
        let (last, time) = self.last_verify.get();
        let now = self.started.elapsed();
        if last == 0
            || bytes == total
            || bytes - last >= 64 * 1024 * 1024
            || now - time >= Duration::from_millis(100)
        {
            self.last_verify.set((bytes, now));
            self.emit("verifying", total, None, None);
        }
    }
    pub fn output_error(&self, mut error: Failure) -> Failure {
        if let Some(io) = self.error.borrow_mut().take() {
            error.code = "output";
            error.message = format!("write progress: {io}");
            error.os_error = io.raw_os_error();
        }
        error
    }
}
pub(crate) struct Combined<'a, C> {
    pub external: &'a C,
    pub output_failed: &'a AtomicBool,
}
impl<C: Cancellation> Cancellation for Combined<'_, C> {
    fn is_cancelled(&self) -> bool {
        self.external.is_cancelled() || self.output_failed.load(Ordering::Acquire)
    }
}
pub(crate) fn check(c: &impl Cancellation) -> Result<(), Failure> {
    if c.is_cancelled() {
        Err(Failure::new("cancelled", "operation cancelled"))
    } else {
        Ok(())
    }
}
