use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version = "0.1.0", about = "TODO")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Process(ProcessArgs),
    Completions(CompletionsArgs),
}

/// Process lines
#[derive(Args, Debug)]
pub struct ProcessArgs {
    /// A regular expression with named matches
    #[arg(short, long)]
    pub pattern: Vec<String>,

    /// A format string
    pub output_template: String,

    /// Input files
    pub files: Vec<String>,
}

/// Print completions for a given shell to stdout
#[derive(Args, Debug)]
pub struct CompletionsArgs {
    /// The name of the shell to generate completions for
    pub shell: clap_complete::Shell,
}
