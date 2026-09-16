use rvvdk_datamover::{CopyOptions, DataMover, ExecutionStrategy};

#[test]
fn datamover_defaults_to_threaded_execution() {
    let options = CopyOptions::new(1024 * 1024).unwrap();

    let mover = DataMover::new(options);

    assert_eq!(mover.execution_strategy(), ExecutionStrategy::Threaded,);
}

#[cfg(target_os = "linux")]
#[test]
fn datamover_accepts_io_uring_execution() {
    use rvvdk_datamover::IoUringExecutionOptions;

    let options = CopyOptions::new(1024 * 1024).unwrap();

    let io_uring = IoUringExecutionOptions::new(8).unwrap();

    let mover = DataMover::with_execution_strategy(options, ExecutionStrategy::IoUring(io_uring));

    assert_eq!(
        mover.execution_strategy(),
        ExecutionStrategy::IoUring(io_uring,),
    );
}

#[cfg(target_os = "linux")]
#[test]
fn datamover_accepts_auto_execution() {
    use rvvdk_datamover::IoUringExecutionOptions;

    let copy_options = CopyOptions::new(1024 * 1024).unwrap();

    let io_uring_options = IoUringExecutionOptions::new(8).unwrap();

    let mover =
        DataMover::with_execution_strategy(copy_options, ExecutionStrategy::Auto(io_uring_options));

    assert_eq!(
        mover.execution_strategy(),
        ExecutionStrategy::Auto(io_uring_options,),
    );
}
