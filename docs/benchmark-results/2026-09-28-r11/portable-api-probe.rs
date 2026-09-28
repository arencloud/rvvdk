use rvvdk_core::{MemoryBlockDevice, RawDisk, Result, VirtualDisk};
use rvvdk_datamover::{CopyOptions, DataMover, ProgressObserver, ProgressSnapshot};

fn main() -> Result<()> {
    let source = RawDisk::new(MemoryBlockDevice::new(4096)?);
    let destination = RawDisk::new(MemoryBlockDevice::new(4096)?);
    let source: &dyn VirtualDisk = &source;
    let destination: &dyn VirtualDisk = &destination;
    source.write_all_at(0, b"portable")?;
    let mover = DataMover::new(CopyOptions::new(4096)?);
    let plan = mover.plan_with_destination(source, destination)?;
    let callback = |_: &ProgressSnapshot| {};
    let observer: &dyn ProgressObserver = &callback;
    mover.execute_plan_with_observer(&plan, source, destination, observer)?;
    let mut actual = [0; 8];
    destination.read_exact_at(0, &mut actual)?;
    assert_eq!(&actual, b"portable");
    println!("portable trait-object copy and observer passed");
    Ok(())
}
