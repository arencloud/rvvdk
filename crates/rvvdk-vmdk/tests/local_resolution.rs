#![cfg(target_os = "linux")]
use rvvdk_vmdk::{
    BackingError, BackingResolver, Descriptor, Limits, LocalResolver, ResolutionLimits,
    ResolvedDescriptor,
};
use std::{
    fs::{self, File},
    os::unix::fs::{MetadataExt, symlink},
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::var_os("RVVDK_TEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let path = root.join(format!(
            "rvvdk-r42-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn resolver(&self) -> LocalResolver {
        LocalResolver::from_directory(File::open(&self.0).unwrap()).unwrap()
    }
    fn write(&self, name: &str, bytes: &[u8]) {
        fs::write(self.0.join(name), bytes).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn text(extents: &str) -> String {
    format!("version=1\nCID=00112233\nparentCID=ffffffff\ncreateType=\"custom\"\n{extents}\n")
}

#[test]
fn descriptor_parent_and_nested_regular_files_are_used_read_only() {
    let f = Fixture::new();
    fs::create_dir(f.0.join("nested")).unwrap();
    f.write("nested/Data café", &[7; 2048]);
    f.write(
        "disk.vmdk",
        text("RW 1 FLAT \"nested/Data café\" 1\nRW 1 ZERO").as_bytes(),
    );
    let (owned, r) =
        LocalResolver::open_descriptor(f.0.join("disk.vmdk"), Limits::default()).unwrap();
    let d = ResolvedDescriptor::resolve(&owned.parse().unwrap(), &r, ResolutionLimits::default())
        .unwrap();
    assert!(
        !d.backings()[0]
            .initial_endpoint()
            .capabilities
            .contains(rvvdk_core::Capabilities::WRITE)
    );
    assert!(d.backings()[0].initial_endpoint().identity.is_some());
    let mut bytes = [0; 512];
    d.backings()[0].read_exact_at(512, &mut bytes).unwrap();
    assert_eq!(bytes, [7; 512]);
    assert_eq!(fs::read(f.0.join("nested/Data café")).unwrap(), [7; 2048]);
}
#[test]
fn dangerous_lexical_names_and_symlink_components_fail() {
    let f = Fixture::new();
    f.write("data", &[1; 512]);
    fs::create_dir(f.0.join("dir")).unwrap();
    symlink("data", f.0.join("link")).unwrap();
    symlink("dir", f.0.join("linkdir")).unwrap();
    symlink("/etc/passwd", f.0.join("outside")).unwrap();
    let r = f.resolver();
    for name in [
        "",
        "/etc/passwd",
        "../data",
        "dir/../../data",
        "./data",
        "dir/../data",
        "dir//data",
        "data/",
        "C:disk",
        "x\\y",
        "x\0y",
        "x\ny",
    ] {
        assert!(
            matches!(r.resolve(name), Err(BackingError::UnsafeReference)),
            "{name:?}"
        );
    }
    for name in ["link", "linkdir/data", "outside"] {
        assert!(
            matches!(r.resolve(name), Err(BackingError::Io { .. })),
            "{name}"
        );
    }
    symlink("data", f.0.join("descriptor")).unwrap();
    assert!(LocalResolver::open_descriptor(f.0.join("descriptor"), Limits::default()).is_err());
}
#[test]
fn nonregular_files_reject_without_opening_a_fifo_for_io() {
    let f = Fixture::new();
    let fifo = std::ffi::CString::new(f.0.join("fifo").as_os_str().as_encoded_bytes()).unwrap();
    // SAFETY: live C string; creates a FIFO in this test's owned directory.
    assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
    let _socket = std::os::unix::net::UnixListener::bind(f.0.join("socket")).unwrap();
    let r = f.resolver();
    for name in ["fifo", "socket"] {
        assert!(matches!(r.resolve(name), Err(BackingError::NotRegular)));
    }
    fs::create_dir(f.0.join("directory")).unwrap();
    assert!(matches!(
        r.resolve("directory"),
        Err(BackingError::NotRegular)
    ));
    f.write("regular", &[0; 512]);
    assert!(matches!(
        LocalResolver::from_directory(File::open(f.0.join("regular")).unwrap()),
        Err(BackingError::NotDirectory)
    ));
}
#[test]
fn mounted_namespaces_and_magic_links_are_not_followed() {
    let r = LocalResolver::from_directory(File::open("/").unwrap()).unwrap();
    assert!(
        matches!(r.resolve("proc/version"), Err(BackingError::Io { source, .. }) if source.raw_os_error() == Some(libc::EXDEV))
    );
    let r = LocalResolver::from_directory(File::open("/proc/self/fd").unwrap()).unwrap();
    let f = File::open("/etc/hosts").unwrap();
    use std::os::fd::AsRawFd;
    assert!(
        matches!(r.resolve(&f.as_raw_fd().to_string()), Err(BackingError::Io { source, .. }) if source.raw_os_error() == Some(libc::ELOOP))
    );
}
#[test]
fn pinned_anchor_and_backings_survive_path_replacement_and_hardlinks_have_identity() {
    let f = Fixture::new();
    fs::create_dir(f.0.join("anchor")).unwrap();
    f.write("anchor/data", &[3; 1024]);
    fs::hard_link(f.0.join("anchor/data"), f.0.join("anchor/alias")).unwrap();
    let r = LocalResolver::from_directory(File::open(f.0.join("anchor")).unwrap()).unwrap();
    fs::rename(f.0.join("anchor"), f.0.join("moved")).unwrap();
    fs::create_dir(f.0.join("anchor")).unwrap();
    f.write("anchor/data", &[9; 1024]);
    let t = text("RW 1 FLAT \"data\" 0\nRW 1 FLAT \"alias\" 1");
    let d = ResolvedDescriptor::resolve(
        &Descriptor::parse(t.as_bytes()).unwrap(),
        &r,
        ResolutionLimits::default(),
    )
    .unwrap();
    assert_eq!(
        d.backings()[0].initial_endpoint().identity,
        d.backings()[1].initial_endpoint().identity
    );
    fs::remove_file(f.0.join("moved/data")).unwrap();
    symlink("/etc/passwd", f.0.join("moved/data")).unwrap();
    d.revalidate().unwrap();
    let mut bytes = [0; 512];
    d.backings()[0].read_exact_at(0, &mut bytes).unwrap();
    assert_eq!(bytes, [3; 512]);
}
#[test]
fn live_truncation_is_detected_and_failed_resolution_leaks_no_file_handles() {
    let f = Fixture::new();
    f.write("data", &[3; 1024]);
    let r = f.resolver();
    let t = text("RW 1 FLAT \"data\" 1");
    let parsed = Descriptor::parse(t.as_bytes()).unwrap();
    let d = ResolvedDescriptor::resolve(&parsed, &r, ResolutionLimits::default()).unwrap();
    File::options()
        .write(true)
        .open(f.0.join("data"))
        .unwrap()
        .set_len(1023)
        .unwrap();
    assert!(matches!(
        d.revalidate(),
        Err(BackingError::Truncated {
            required: 1024,
            actual: 1023,
            ..
        })
    ));
    assert!(d.backings()[0].read_exact_at(512, &mut [0; 512]).is_err());
    drop(d);
    let metadata = fs::metadata(f.0.join("data")).unwrap();
    for _ in 0..30 {
        assert!(ResolvedDescriptor::resolve(&parsed, &r, ResolutionLimits::default()).is_err());
    }
    let retained = fs::read_dir("/proc/self/fd")
        .unwrap()
        .filter_map(Result::ok)
        .filter_map(|e| fs::metadata(e.path()).ok())
        .filter(|m| (m.dev(), m.ino()) == (metadata.dev(), metadata.ino()))
        .count();
    assert_eq!(retained, 0);
}
#[test]
fn descriptor_acquisition_ignores_untrusted_file_size_and_obeys_bound() {
    let f = Fixture::new();
    let t = text("RW 1 ZERO");
    f.write("disk", t.as_bytes());
    let limits = Limits {
        descriptor_bytes: t.len(),
        ..Limits::default()
    };
    assert!(LocalResolver::open_descriptor(f.0.join("disk"), limits).is_ok());
    f.write("disk", &[b' '; 8192]);
    assert!(matches!(
        LocalResolver::open_descriptor(f.0.join("disk"), limits),
        Err(BackingError::Limit(_))
    ));
}
