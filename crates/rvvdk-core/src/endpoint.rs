use crate::Capabilities;

/// Identity of the current backing object, not a content/snapshot fingerprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointIdentity {
    LocalFile {
        device: u64,
        inode: u64,
    },
    /// Process-local identity valid while the device is borrowed. Never persist it.
    Memory {
        address: usize,
    },
}

/// Current endpoint facts used before a copy. Backends should refresh mutable
/// size/access facts and forward identity through wrappers. `None` means unknown,
/// not proof that endpoints are distinct. This is not a lease against mutation.
#[derive(Debug, Clone, Copy)]
pub struct CopyEndpoint {
    pub size: u64,
    pub capabilities: Capabilities,
    pub identity: Option<EndpointIdentity>,
}
