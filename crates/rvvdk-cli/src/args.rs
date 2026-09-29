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
                .value_parser(["raw"])
                .help("Explicit input format; no format autodetection"),
        )
        .arg(
            Arg::new("extents")
                .long("extents")
                .action(ArgAction::SetTrue)
                .help("Include the complete logical extent map"),
        )
        .arg(
            Arg::new("memory-budget")
                .long("memory-budget")
                .default_value("268435456")
                .value_parser(value_parser!(usize))
                .help("Planning/execution payload budget in bytes; not an RSS limit"),
        )
}
pub(crate) fn command() -> Command {
    Command::new("rvddk").version(env!("CARGO_PKG_VERSION"))
        .about("Inspect local RAW disks and preview copy intent without modifying files")
        .subcommand_required(true).arg_required_else_help(true)
        .arg(Arg::new("json").long("json").global(true).action(ArgAction::SetTrue)
            .help("Emit schema-versioned JSON; errors go to stderr"))
        .subcommand(source(Command::new("inspect").about("Inspect a regular RAW file")))
        .subcommand(source(Command::new("plan").about("Preview logical work and destination policy; does not create or write output"))
            .arg(Arg::new("destination").required(true).value_parser(value_parser!(PathBuf)))
            .arg(Arg::new("overwrite").long("overwrite").action(ArgAction::SetTrue)
                .help("Preview in-place overwrite of an existing file; still performs no writes"))
            .arg(Arg::new("backend").long("backend").default_value("threaded")
                .value_parser(["threaded", "auto", "io-uring"])
                .help("Requested copy backend; native selection is deferred until destination preparation"))
            .arg(Arg::new("block-size").long("block-size").default_value("1048576")
                .value_parser(positive).help("Copy block size in bytes"))
            .arg(Arg::new("workers").long("workers").default_value("1")
                .value_parser(positive).help("Threaded worker count"))
            .arg(Arg::new("queue-depth").long("queue-depth").default_value("8")
                .value_parser(value_parser!(u32).range(1..)).help("Requested native queue depth")))
}
