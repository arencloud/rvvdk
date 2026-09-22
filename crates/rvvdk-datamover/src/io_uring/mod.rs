mod capabilities;
mod compatibility;
mod copy;
mod engine;
mod extent_copy;
mod extent_plan;
mod operation;

pub use capabilities::{IoUringCapabilities, probe_io_uring};

pub use engine::IoUringEngine;

pub use operation::{CompletedOperation, IoUringOperationKind};

pub use copy::{IoUringCopyStats, copy_file_range, copy_file_range_with_options};

pub use compatibility::{IoUringCompatibility, evaluate_compatibility};
pub use extent_copy::{
    IoUringExtentCopyStats, copy_extent_plan, copy_extent_plan_with_destination,
};
pub use extent_plan::NativeExtentPlan;
