//! Linux RAW inspection and read-only plan previews for the rvddk command.
//! The JSON report is not a serialized executable CopyPlan.
mod args;
mod error;
mod output;
mod preview;

use error::Failure;
use std::{ffi::OsString, io::Write};

/// Run the CLI with supplied arguments and streams. Exit codes: 0 success/help,
/// 1 inspection/planning/output failure, 2 command-line usage error.
pub fn run<I, T>(arguments: I, out: &mut impl Write, err: &mut impl Write) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let arguments: Vec<OsString> = arguments.into_iter().map(Into::into).collect();
    let json = arguments
        .iter()
        .skip(1)
        .take_while(|a| *a != "--")
        .any(|a| a == "--json");
    let matches = match args::command().try_get_matches_from(arguments) {
        Ok(matches) => matches,
        Err(error) => {
            if !error.use_stderr() {
                return if write!(out, "{error}").and_then(|_| out.flush()).is_ok() {
                    0
                } else {
                    1
                };
            }
            emit_error(&Failure::new("usage", error.to_string()), json, err);
            return 2;
        }
    };
    let (name, args) = matches.subcommand().expect("required subcommand");
    match preview::build(name, args).and_then(|report| output::write(&report, json, out)) {
        Ok(()) => 0,
        Err(error) => {
            emit_error(&error, json, err);
            1
        }
    }
}
fn emit_error(error: &Failure, json: bool, err: &mut impl Write) {
    if json {
        let report = serde_json::json!({"schema_version": 1, "error": error});
        let _ = serde_json::to_writer(&mut *err, &report);
        let _ = writeln!(err);
    } else {
        let _ = writeln!(err, "rvddk: {}: {}", error.code, error.message);
    }
    let _ = err.flush();
}
