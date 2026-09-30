use clap::{Arg, ArgAction, Command, value_parser};
use std::path::PathBuf;

fn positive(value: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .ok()
        .filter(|&v| v > 0)
        .ok_or_else(|| "expected a positive byte/count value in decimal".into())
}
fn source(command: Command) -> Command {
    command
        .arg(
            Arg::new("source")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .required(true)
                .value_parser(["raw", "vmdk"])
                .help("Explicit input format; no format autodetection"),
        )
        .arg(Arg::new("allow-parents").long("allow-parents").action(ArgAction::SetTrue)
            .help("Opt in to hosted-sparse parent chains in the source directory; requires --format vmdk"))
        .arg(
            Arg::new("memory-budget")
                .long("memory-budget")
                .default_value("268435456")
                .value_parser(value_parser!(usize))
                .help("Planning/execution payload budget in bytes; not an RSS limit"),
        )
}
fn extents(command: Command) -> Command {
    command.arg(
        Arg::new("extents")
            .long("extents")
            .action(ArgAction::SetTrue)
            .help("Include the complete logical extent map"),
    )
}
fn block_size(command: Command) -> Command {
    command.arg(
        Arg::new("block-size")
            .long("block-size")
            .default_value("1048576")
            .value_parser(positive)
            .help("Copy/comparison block size in bytes"),
    )
}
fn destination(command: Command) -> Command {
    command.arg(
        Arg::new("destination")
            .required(true)
            .value_parser(value_parser!(PathBuf)),
    )
}
fn copy_options(command: Command) -> Command {
    block_size(destination(source(command)))
        .arg(
            Arg::new("overwrite")
                .long("overwrite")
                .action(ArgAction::SetTrue)
                .help(
                    "Modify an existing destination in place; failures may leave partial changes",
                ),
        )
        .arg(
            Arg::new("backend")
                .long("backend")
                .default_value("threaded")
                .value_parser(["threaded", "auto", "io-uring"])
                .help("Execution backend; Auto can fall back before mutation"),
        )
        .arg(
            Arg::new("workers")
                .long("workers")
                .default_value("1")
                .value_parser(positive)
                .help("Threaded worker count"),
        )
        .arg(
            Arg::new("queue-depth")
                .long("queue-depth")
                .default_value("8")
                .value_parser(value_parser!(u32).range(1..))
                .help("Requested native queue depth"),
        )
}
fn progress(command: Command) -> Command {
    command.arg(
        Arg::new("progress")
            .long("progress")
            .action(ArgAction::SetTrue)
            .help("Write lifecycle progress to stderr; JSON lines with --json"),
    )
}
pub(crate) fn command() -> Command {
    // Build only the selected command's options. The full list and summaries
    // remain available in root help, without cloning unrelated argument trees.
    Command::new("rvddk").version(env!("CARGO_PKG_VERSION"))
        .about("Inspect, plan, copy, and verify local RAW or FLAT/ZERO/hosted-sparse VMDK sources (RAW destinations)")
        .subcommand_required(true).arg_required_else_help(true)
        .arg(Arg::new("json").long("json").global(true).action(ArgAction::SetTrue)
            .help("Emit schema-versioned JSON; errors go to stderr"))
        .subcommand(Command::new("inspect").about("Inspect a RAW file, base VMDK descriptor or hosted sparse container")
            .defer(|c| extents(source(c))))
        .subcommand(Command::new("plan").about("Preview logical work and destination policy; does not create or write output")
            .defer(|c| extents(copy_options(c))
                .mut_arg("overwrite", |a| a.help("Preview in-place overwrite of an existing file; still performs no writes"))
                .mut_arg("backend", |a| a.help("Requested copy backend; native selection is deferred until destination preparation"))))
        .subcommand(Command::new("copy").about("Copy logical bytes to RAW; publish new output without replacement")
            .defer(|c| progress(copy_options(c)).arg(Arg::new("verify").long("verify").action(ArgAction::SetTrue)
                .help("Read back and compare all logical bytes before publishing new output"))))
        .subcommand(Command::new("verify").about("Compare source bytes with the destination prefix; no writes")
            .defer(|c| progress(block_size(destination(source(c))))))
}
