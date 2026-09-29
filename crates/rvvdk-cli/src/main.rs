fn main() -> std::process::ExitCode {
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    let mut err = std::io::stderr().lock();
    std::process::ExitCode::from(rvvdk_cli::run(std::env::args_os(), &mut out, &mut err) as u8)
}
