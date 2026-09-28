// Diagnostic harness: copy to crates/rvvdk-datamover/examples/preflight_probe.rs.
// Pass existing distinct regular source/destination files of equal size.
// Markers delimit exactly 100 planning calls in strace's write/metadata trace.
use rvvdk_core::RawDisk;
use rvvdk_datamover::{CopyOptions, DataMover, ExecutionStrategy, IoUringExecutionOptions};
use rvvdk_local::LocalFileBlockDevice;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let source = RawDisk::new(LocalFileBlockDevice::open_read_only(&args[1]).unwrap());
    let destination = RawDisk::new(LocalFileBlockDevice::open_read_write(&args[2]).unwrap());
    for strategy in [
        ExecutionStrategy::Threaded,
        ExecutionStrategy::IoUring(IoUringExecutionOptions::new(8).unwrap()),
    ] {
        let mover = DataMover::with_execution_strategy(CopyOptions::new(65536).unwrap(), strategy);
        eprintln!("BEGIN {strategy:?}");
        for _ in 0..100 {
            std::hint::black_box(mover.plan_raw_with_destination(&source, &destination).unwrap());
        }
        eprintln!("END");
    }
}
