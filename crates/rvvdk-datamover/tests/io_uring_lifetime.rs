#![cfg(target_os = "linux")]

use std::fs::{File, OpenOptions};
use std::os::fd::AsFd;
use std::process::Command;
use std::time::{Duration, Instant};

use rvvdk_core::Error;
use rvvdk_datamover::io_uring::copy_file_range;

fn bounded(name: &str, test: impl FnOnce()) {
    const CHILD: &str = "RVVDK_PIPELINE_LIFETIME_CHILD";
    if std::env::var(CHILD).as_deref() == Ok(name) {
        test();
        return;
    }
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture"])
        .env(CHILD, name)
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            return;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("pipeline error cleanup exceeded ten seconds");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn fd_count() -> usize {
    std::fs::read_dir("/proc/self/fd").unwrap().count()
}

#[test]
fn eof_cleanup_preserves_original_error_and_releases_descriptors() {
    bounded(
        "eof_cleanup_preserves_original_error_and_releases_descriptors",
        || {
            let path = std::env::temp_dir().join(format!("rvvdk-r03-eof-{}", std::process::id()));
            let source = OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .open(&path)
                .unwrap();
            std::fs::remove_file(&path).unwrap();
            let destination = OpenOptions::new().write(true).open("/dev/null").unwrap();
            let before = fd_count();
            for _ in 0..16 {
                let error = copy_file_range(
                    source.as_fd(),
                    destination.as_fd(),
                    0,
                    64 * 1024,
                    4096,
                    8,
                    4096,
                )
                .unwrap_err();
                assert!(matches!(error, Error::UnexpectedEof { .. }));
                assert_eq!(fd_count(), before);
            }
        },
    );
}

#[test]
fn write_error_cleanup_preserves_errno_and_releases_descriptors() {
    bounded(
        "write_error_cleanup_preserves_errno_and_releases_descriptors",
        || {
            let source = File::open("/dev/zero").unwrap();
            let destination = File::open("/dev/null").unwrap();
            let before = fd_count();
            for _ in 0..16 {
                let error = copy_file_range(
                    source.as_fd(),
                    destination.as_fd(),
                    0,
                    64 * 1024,
                    4096,
                    8,
                    4096,
                )
                .unwrap_err();
                assert!(matches!(error, Error::Io(ref cause) if cause.raw_os_error() == Some(9)));
                assert_eq!(fd_count(), before);
            }
        },
    );
}
