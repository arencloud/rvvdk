use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Capabilities: u64 {
        const READ        = 1 << 0;
        const WRITE       = 1 << 1;
        const FLUSH       = 1 << 2;
        /// Accepts discard requests; success alone does not guarantee zero reads.
        const DISCARD     = 1 << 3;
        /// Successful write_zero_at makes the complete requested range read zero.
        const WRITE_ZERO  = 1 << 4;
        const EXTENTS     = 1 << 5;
        const SPARSE      = 1 << 6;
        const DIRECT_IO   = 1 << 7;
        /// With DISCARD, successful discard makes the entire requested logical
        /// range read zero, preserves bytes outside it, and preserves disk size.
        /// This modifier alone does not advertise an operation. It guarantees
        /// logical contents, not physical space reclamation or durability.
        const DISCARD_ZEROES = 1 << 8;
    }
}
