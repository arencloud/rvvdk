#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::{FileInspection, FileState, LinuxFdBackend, LinuxFdCapabilities, inspect_file};
