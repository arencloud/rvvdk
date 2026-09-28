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
fn short_source_preflight_preserves_context_without_leaking_descriptors() {
    bounded(
        "short_source_preflight_preserves_context_without_leaking_descriptors",
        || {
            let path = std::env::temp_dir().join(format!("rvvdk-r03-eof-{}", std::process::id()));
            let source = OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .open(&path)
                .unwrap();
            std::fs::remove_file(&path).unwrap();
            let destination = regular_file("destination", true);
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
                assert!(
                    matches!(error, Error::EndpointPreflight { endpoint: "source", source } if matches!(*source, Error::OutOfBounds { .. }))
                );
                assert_eq!(fd_count(), before);
            }
        },
    );
}

#[test]
fn readonly_preflight_preserves_context_without_leaking_descriptors() {
    bounded(
        "readonly_preflight_preserves_context_without_leaking_descriptors",
        || {
            let source = regular_file("source", true);
            let destination = regular_file("destination", false);
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
                assert!(
                    matches!(error, Error::EndpointPreflight { endpoint: "destination", source } if matches!(*source, Error::MissingCapability { capability: "write" }))
                );
                assert_eq!(fd_count(), before);
            }
        },
    );
}

fn regular_file(name: &str, writable: bool) -> File {
    let path =
        std::env::temp_dir().join(format!("rvvdk-r05-lifetime-{}-{name}", std::process::id()));
    let created = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    created.set_len(64 * 1024).unwrap();
    let file = OpenOptions::new()
        .read(true)
        .write(writable)
        .open(&path)
        .unwrap();
    std::fs::remove_file(path).unwrap();
    file
}
