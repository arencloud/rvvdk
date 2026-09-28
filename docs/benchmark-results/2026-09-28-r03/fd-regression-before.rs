#![cfg(target_os = "linux")]
use std::os::fd::AsRawFd;
use std::os::unix::fs::FileExt;
use rvvdk_core::BufferPool;
use rvvdk_datamover::io_uring::IoUringEngine;
#[test]
fn queued_descriptor_must_survive_caller_close() {
 let path=std::env::temp_dir().join(format!("rvvdk-r03-before-{}",std::process::id()));
 let file=std::fs::OpenOptions::new().read(true).write(true).create_new(true).open(&path).unwrap();
 std::fs::remove_file(&path).unwrap();
 file.write_all_at(&vec![0x5a;4096],0).unwrap();
 let fd=file.as_raw_fd();
 let pool=BufferPool::new(1,4096,4096).unwrap();
 let mut engine=IoUringEngine::new(1).unwrap();
 engine.submit_owned_read(fd,0,4096,pool.acquire()).unwrap();
 drop(file);
 let replacement=std::fs::File::open("/dev/zero").unwrap();
 assert_eq!(replacement.as_raw_fd(),fd);
 let completed=engine.wait_owned_completion().unwrap();
 assert!(completed.buffer().iter().all(|b| *b==0x5a),"queued read used a recycled descriptor");
}
