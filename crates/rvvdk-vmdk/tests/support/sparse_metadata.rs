#[path = "sparse.rs"]
mod raw;
use rvvdk_core::{BlockDevice, MemoryBlockDevice};
use rvvdk_vmdk::{BackingError, BackingResolver};
use std::sync::Arc;
pub fn descriptor(kind: &str, extents: &str) -> String {
    format!("version=1\nCID=12345678\nparentCID=ffffffff\ncreateType=\"{kind}\"\n{extents}\n")
}
pub fn bytes(text: Option<&str>) -> Vec<u8> {
    let mut b = vec![0; 196608];
    b[..512].copy_from_slice(&raw::header());
    if let Some(t) = text {
        b[512..512 + t.len()].copy_from_slice(t.as_bytes());
    }
    raw::put32(&mut b, 21 * 512, 22);
    raw::put32(&mut b, 26 * 512, 27);
    for table in [22, 27] {
        raw::put32(&mut b, table * 512, 128);
        raw::put32(&mut b, table * 512 + 4, 256);
    }
    b[65536..131072].fill(0x5a);
    b[131072..].fill(0xa5);
    b
}
pub fn put32(b: &mut [u8], offset: usize, value: u32) {
    raw::put32(b, offset, value)
}
pub struct Resolver(pub Arc<dyn BlockDevice>);
impl BackingResolver for Resolver {
    fn resolve(&self, name: &str) -> Result<Arc<dyn BlockDevice>, BackingError> {
        assert_eq!(name, "disk.vmdk");
        Ok(self.0.clone())
    }
}
pub fn device(bytes: &[u8]) -> Arc<MemoryBlockDevice> {
    let d = Arc::new(MemoryBlockDevice::new(bytes.len()).unwrap());
    d.write_all_at(0, bytes).unwrap();
    d
}
