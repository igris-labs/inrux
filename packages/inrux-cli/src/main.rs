//! Cargo subcommand; compilation and dependency resolution remain Cargo's responsibility.
mod commands;
mod dev;
mod preview;
mod scaffold;
use std::process::ExitCode;

/// Preserve child exit codes so editor/CI callers can reliably detect failed checks.
fn main() -> ExitCode {
    match commands::run(std::env::args().skip(1).collect()) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("inrux: {error}");
            ExitCode::FAILURE
        }
    }
}
