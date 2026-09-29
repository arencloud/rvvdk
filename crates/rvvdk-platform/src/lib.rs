#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::{FileInspection, FileState, LinuxFdBackend, LinuxFdCapabilities, inspect_file};

#[cfg(target_os = "linux")]
mod access;
#[cfg(target_os = "linux")]
pub use access::{FileAccess, FileAccessGuard, FileAccessKind};
