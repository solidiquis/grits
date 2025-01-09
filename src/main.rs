use anyhow::Result;
use clap::{crate_name, CommandFactory, Parser};
use std::{env, process::ExitCode};

mod cli;
use cli::{Cli, Command};

mod line;
mod scanner;
mod template;

mod tty;
use tty::TtyContext;

fn main() -> ExitCode {
    if env::var("RUST_LOG").is_ok() {
        env_logger::init();
    }

    let ctx = Cli::parse();
    let mut tty = TtyContext::new();

    if let Err(e) = run(&ctx, &mut tty) {
        log::error!("{e:?}");
        let _ = tty.write_err(&format!("{e:?}"));
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run(cli: &Cli, tty: &mut TtyContext) -> Result<()> {
    match &cli.command {
        Command::Process(args) => line::run(tty, args),
        Command::Completions(args) => {
            clap_complete::generate(
                args.shell,
                &mut Cli::command(),
                crate_name!(),
                &mut tty.stdout,
            );
            Ok(())
        }
    }
}
