use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Capabilities: u64 {
        const READ        = 1 << 0;
        const WRITE       = 1 << 1;
        const FLUSH       = 1 << 2;
        const DISCARD     = 1 << 3;
        const WRITE_ZERO  = 1 << 4;
        const EXTENTS     = 1 << 5;
        const SPARSE      = 1 << 6;
        const DIRECT_IO   = 1 << 7;
    }
}
