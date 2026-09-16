use rvvdk_datamover::io_uring::probe_io_uring;

fn main() {
    let capabilities = probe_io_uring(32).expect("io_uring probe failed");

    println!("io_uring queue depth: {}", capabilities.queue_depth(),);

    println!("IORING_FEAT_FAST_POLL: {}", capabilities.fast_poll(),);

    println!("IORING_FEAT_NODROP: {}", capabilities.nodrop(),);

    println!(
        "IORING_FEAT_SUBMIT_STABLE: {}",
        capabilities.submit_stable(),
    );
}
