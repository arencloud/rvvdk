use thiserror::Error;

/// A deterministic native request restriction, detected before submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum NativeRequestIssue {
    #[error("native block size {block_size} must be in 1..=u32::MAX")]
    BlockSize { block_size: usize },
    #[error("native range offset={offset}, length={length} exceeds the nonnegative i64 domain")]
    Range { offset: u64, length: u64 },
    #[error("{endpoint} declares invalid native alignment: memory={memory}, offset={offset}")]
    DescriptorAlignment {
        endpoint: &'static str,
        memory: usize,
        offset: usize,
    },
    #[error("invalid native buffer layout: block_size={block_size}, alignment={alignment}")]
    BufferLayout { block_size: usize, alignment: usize },
    #[error(
        "{endpoint} direct request offset={offset}, length={length} requires alignment={alignment}"
    )]
    DirectRange {
        endpoint: &'static str,
        offset: u64,
        length: u64,
        alignment: usize,
    },
    #[error("{endpoint} native block size={block_size} requires direct alignment={alignment}")]
    DirectBlockSize {
        endpoint: &'static str,
        block_size: usize,
        alignment: usize,
    },
}
