use rvvdk_core::{
    BlockDevice, Capabilities, CopyEndpoint, DiskGeometry, EndpointIdentity, MemoryBlockDevice,
};
use rvvdk_vmdk::{
    BackingError, BackingResolver, Descriptor, DescriptorText, Limits, ResolutionLimits,
    ResolvedDescriptor, ResolvedExtentBacking,
};
use std::{
    io::{self, Read},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
fn text(extents: &str) -> String {
    format!("version=1\nCID=00112233\nparentCID=ffffffff\ncreateType=\"custom\"\n{extents}\n")
}
struct Resolver {
    calls: AtomicUsize,
    device: Arc<dyn BlockDevice>,
}
impl BackingResolver for Resolver {
    fn resolve(&self, name: &str) -> Result<Arc<dyn BlockDevice>, BackingError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if name == "missing" {
            return Err(BackingError::UnsafeReference);
        }
        Ok(self.device.clone())
    }
}
fn resolver(size: usize) -> Resolver {
    Resolver {
        calls: AtomicUsize::new(0),
        device: Arc::new(MemoryBlockDevice::read_only(size).unwrap()),
    }
}

#[test]
fn opaque_references_deduplicate_and_outlive_input_and_resolver() {
    let r = resolver(4096);
    let t = text("RDONLY 1 FLAT \"memory:object\" 1\nRW 2 ZERO\nRW 2 FLAT \"memory:object\" 4");
    let result = ResolvedDescriptor::resolve(
        &Descriptor::parse(t.as_bytes()).unwrap(),
        &r,
        ResolutionLimits::default(),
    )
    .unwrap();
    assert_eq!(r.calls.load(Ordering::SeqCst), 1);
    assert_eq!(result.backings().len(), 1);
    assert_eq!(result.backings()[0].required_len(), 3072);
    assert_eq!(result.extents()[2].logical_offset(), 1536);
    assert_eq!(
        result.extents()[2].backing(),
        ResolvedExtentBacking::Flat {
            backing_index: 0,
            offset_bytes: 2048
        }
    );
    assert_eq!(result.extents()[1].backing(), ResolvedExtentBacking::Zero);
    assert_eq!(result.size_bytes(), 2560);
    drop(t);
    drop(r);
    result.revalidate().unwrap();
    let mut bytes = [1; 512];
    result.backings()[0]
        .read_exact_at(2048, &mut bytes)
        .unwrap();
    assert_eq!(bytes, [0; 512]);
}

#[test]
fn resource_limits_fail_before_any_resolver_call() {
    let r = resolver(4096);
    let t = text("RW 1 FLAT \"a\" 0\nRW 1 FLAT \"b\" 0");
    let d = Descriptor::parse(t.as_bytes()).unwrap();
    for limits in [
        ResolutionLimits {
            extents: 1,
            backing_files: 2,
        },
        ResolutionLimits {
            extents: 2,
            backing_files: 1,
        },
    ] {
        assert!(matches!(
            ResolvedDescriptor::resolve(&d, &r, limits),
            Err(BackingError::Limit(_))
        ));
        assert_eq!(r.calls.load(Ordering::SeqCst), 0);
    }
    let d = ResolvedDescriptor::resolve(
        &d,
        &r,
        ResolutionLimits {
            extents: 2,
            backing_files: 2,
        },
    )
    .unwrap();
    assert_eq!(d.backings().len(), 2);
}
#[test]
fn zero_extents_resolve_without_sources() {
    let t = text("RW 8 ZERO");
    let r = resolver(0);
    let d = ResolvedDescriptor::resolve(
        &Descriptor::parse(t.as_bytes()).unwrap(),
        &r,
        ResolutionLimits {
            extents: 1,
            backing_files: 0,
        },
    )
    .unwrap();
    assert_eq!(d.size_bytes(), 4096);
    assert!(d.backings().is_empty());
    d.revalidate().unwrap();
    assert_eq!(r.calls.load(Ordering::SeqCst), 0);
}
#[test]
fn truncation_and_read_permissions_are_checked_and_failures_drop_sources() {
    let t = text("RW 1 FLAT \"a\" 0\nRW 1 FLAT \"b\" 2");
    let r = resolver(1024);
    assert!(matches!(
        ResolvedDescriptor::resolve(
            &Descriptor::parse(t.as_bytes()).unwrap(),
            &r,
            ResolutionLimits::default()
        ),
        Err(BackingError::Truncated {
            index: 1,
            required: 1536,
            actual: 1024
        })
    ));
    assert_eq!(Arc::strong_count(&r.device), 1);
    let t = text("RW 1 FLAT \"a\" 0\nRW 1 FLAT \"missing\" 0");
    assert!(matches!(
        ResolvedDescriptor::resolve(
            &Descriptor::parse(t.as_bytes()).unwrap(),
            &r,
            ResolutionLimits::default()
        ),
        Err(BackingError::Resolve { index: 1, .. })
    ));
    assert_eq!(Arc::strong_count(&r.device), 1);
    let r = Resolver {
        calls: AtomicUsize::new(0),
        device: Arc::new(MemoryBlockDevice::with_capabilities(4096, Capabilities::WRITE).unwrap()),
    };
    assert!(matches!(
        ResolvedDescriptor::resolve(
            &Descriptor::parse(text("RW 1 FLAT \"a\" 0").as_bytes()).unwrap(),
            &r,
            ResolutionLimits::default()
        ),
        Err(BackingError::Unreadable { index: 0 })
    ));
}
struct Mutable {
    endpoint: Mutex<CopyEndpoint>,
}
impl BlockDevice for Mutable {
    fn geometry(&self) -> DiskGeometry {
        DiskGeometry::new(4096, 512, 4096).unwrap()
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities::READ
    }
    fn copy_endpoint(&self) -> rvvdk_core::Result<CopyEndpoint> {
        Ok(*self.endpoint.lock().unwrap())
    }
    fn read_at(&self, _: u64, _: &mut [u8]) -> rvvdk_core::Result<usize> {
        unreachable!()
    }
    fn write_at(&self, _: u64, _: &[u8]) -> rvvdk_core::Result<usize> {
        unreachable!()
    }
    fn flush(&self) -> rvvdk_core::Result<()> {
        unreachable!()
    }
}
#[test]
fn revalidation_uses_live_capacity_access_and_known_identity() {
    let initial = CopyEndpoint {
        size: 4096,
        capabilities: Capabilities::READ,
        identity: Some(EndpointIdentity::Memory { address: 1 }),
    };
    let device = Arc::new(Mutable {
        endpoint: Mutex::new(initial),
    });
    let r = Resolver {
        calls: AtomicUsize::new(0),
        device: device.clone(),
    };
    let t = text("RW 1 FLAT \"a\" 1");
    let d = ResolvedDescriptor::resolve(
        &Descriptor::parse(t.as_bytes()).unwrap(),
        &r,
        ResolutionLimits::default(),
    )
    .unwrap();
    *device.endpoint.lock().unwrap() = CopyEndpoint {
        size: 1023,
        ..initial
    };
    assert!(matches!(
        d.revalidate(),
        Err(BackingError::Truncated { .. })
    ));
    *device.endpoint.lock().unwrap() = CopyEndpoint {
        capabilities: Capabilities::empty(),
        ..initial
    };
    assert!(matches!(
        d.revalidate(),
        Err(BackingError::Unreadable { .. })
    ));
    for identity in [None, Some(EndpointIdentity::Memory { address: 2 })] {
        *device.endpoint.lock().unwrap() = CopyEndpoint {
            identity,
            ..initial
        };
        assert!(matches!(d.revalidate(), Err(BackingError::Changed { .. })));
    }
    *device.endpoint.lock().unwrap() = CopyEndpoint {
        size: 8192,
        ..initial
    };
    d.revalidate().unwrap();
    assert_eq!(r.calls.load(Ordering::SeqCst), 1);
    *device.endpoint.lock().unwrap() = CopyEndpoint {
        identity: None,
        ..initial
    };
    let unknown = ResolvedDescriptor::resolve(
        &Descriptor::parse(t.as_bytes()).unwrap(),
        &r,
        ResolutionLimits::default(),
    )
    .unwrap();
    assert_eq!(unknown.backings()[0].initial_endpoint().identity, None);
    unknown.revalidate().unwrap();
}

struct ShortReader {
    bytes: Vec<u8>,
    position: usize,
    interrupt: bool,
    fail: bool,
}
impl Read for ShortReader {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.interrupt {
            self.interrupt = false;
            return Err(io::ErrorKind::Interrupted.into());
        }
        if self.fail {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        let count = out.len().min(3).min(self.bytes.len() - self.position);
        out[..count].copy_from_slice(&self.bytes[self.position..self.position + count]);
        self.position += count;
        Ok(count)
    }
}
#[test]
fn bounded_loading_handles_short_reads_interrupts_exact_limit_and_one_probe() {
    let t = text("RW 1 ZERO");
    let limits = Limits {
        descriptor_bytes: t.len(),
        ..Limits::default()
    };
    let mut input = ShortReader {
        bytes: t.as_bytes().to_vec(),
        position: 0,
        interrupt: true,
        fail: false,
    };
    let owned = DescriptorText::read_from(&mut input, limits).unwrap();
    assert_eq!(owned.as_bytes(), t.as_bytes());
    assert_eq!(owned.parse().unwrap().size_bytes(), 512);
    input.position = 0;
    input.bytes.extend_from_slice(&[b' '; 100]);
    assert!(matches!(
        DescriptorText::read_from(&mut input, limits),
        Err(BackingError::Limit("descriptor acquisition bytes"))
    ));
    assert_eq!(input.position, t.len() + 1);
    input.position = 0;
    assert!(matches!(
        DescriptorText::read_from(
            &mut input,
            Limits {
                descriptor_bytes: 0,
                ..limits
            }
        ),
        Err(BackingError::Limit(_))
    ));
    assert_eq!(input.position, 1);
}
#[test]
fn loading_propagates_io_and_parser_failures() {
    let mut input = ShortReader {
        bytes: vec![],
        position: 0,
        interrupt: false,
        fail: true,
    };
    assert!(matches!(
        DescriptorText::read_from(&mut input, Limits::default()),
        Err(BackingError::Io { .. })
    ));
    assert!(matches!(
        DescriptorText::read_from(&b"invalid"[..], Limits::default()),
        Err(BackingError::Descriptor(_))
    ));
}
