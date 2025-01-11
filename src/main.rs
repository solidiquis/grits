use anyhow::Result;
use clap::{crate_name, CommandFactory, Parser};
use std::{env, process::ExitCode};

/// Defines the commandline-interface and the context of the application.
mod cli;
use cli::Cli;

/// Concerned with the actual line-processing.
mod line;

/// Contains iterator types that read in lines from various sources.
mod scanner;

/// Defines the user-sourced template strings that define how to transform input lines and generate
/// an output.
mod template;

/// Contains the terminal context. The rest of the application accesses handlers to standard output
/// and standard error via the [TtyContext]. Also concerned with output colorization and writing to
/// standard output.
mod tty;
use tty::TtyContext;

fn main() -> ExitCode {
    if env::var("RUST_LOG").is_ok() {
        env_logger::init();
    }
    let mut tty = TtyContext::new();

    if let Err(e) = run(&mut tty) {
        log::error!("{e:?}");
        let _ = tty.write_err(&format!("{e:?}"));
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run(tty: &mut TtyContext) -> Result<()> {
    if let Some(shell) = Cli::compute_shell_used_for_completions()? {
        clap_complete::generate(shell, &mut Cli::command(), crate_name!(), &mut tty.stdout);
        return Ok(());
    }
    let ctx = Cli::parse();
    line::process_lines(tty, &ctx)
}
