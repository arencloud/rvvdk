use rvvdk_core::{BlockDevice, MemoryBlockDevice};
use rvvdk_vmdk::{
    BackingError, BackingResolver, Descriptor, ResolutionLimits, ResolvedDescriptor, VmdkDisk,
};
use std::sync::Arc;
pub fn descriptor(extents: &str) -> String {
    format!("version=1\nCID=00112233\nparentCID=ffffffff\ncreateType=\"custom\"\n{extents}\n")
}
pub struct Source(pub Arc<dyn BlockDevice>);
impl BackingResolver for Source {
    fn resolve(&self, _: &str) -> Result<Arc<dyn BlockDevice>, BackingError> {
        Ok(self.0.clone())
    }
}
pub fn patterned() -> (Source, Vec<u8>) {
    let bytes: Vec<_> = (0..8192)
        .map(|i| ((i * 29 + i / 251) % 256) as u8)
        .collect();
    let memory = MemoryBlockDevice::new(bytes.len()).unwrap();
    memory.write_all_at(0, &bytes).unwrap();
    (Source(Arc::new(memory)), bytes)
}
pub fn disk(text: &str, resolver: &dyn BackingResolver) -> VmdkDisk {
    let d = Descriptor::parse(text.as_bytes()).unwrap();
    VmdkDisk::new(ResolvedDescriptor::resolve(&d, resolver, ResolutionLimits::default()).unwrap())
        .unwrap()
}
