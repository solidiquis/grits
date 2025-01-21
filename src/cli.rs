use anyhow::{format_err, Result};
use clap::{crate_authors, crate_version, Parser, ValueEnum};
use clap_complete::Shell;
use std::{env, fmt, str::FromStr};

#[derive(Parser, Debug)]
#[command(
    author = crate_authors!(),
    version = crate_version!(),
    about = "
A text line processor that applies regular expressions with named captures to input lines
and transforms them using a user-generated template. See the long help '--help' for further
details and examples or visit the 'https://github.com/solidiquis/grits' repository.",
    long_about = include_str!("../docs/help.md"),
)]
pub struct Cli {
    /// A regular expression with named captures. Can be specified multiple times.
    #[arg(short, long)]
    pub pattern: Vec<String>,

    /// A template string that defines how to transform a line input using
    /// times. Can be specified multiple times. See long '--help'.
    #[arg(short, long, group = "tmpl")]
    pub template: Vec<String>,

    /// Separator used to join results of transforming each template if multiple are specified.
    #[arg(short, long, default_value_t = String::new(), group="tmpl")]
    pub separator: String,

    /// Input files.
    pub files: Vec<String>,

    /// Comma-separated capture names that must have a match for a given input line to be
    /// processed; otherwise it is ignored.
    #[arg(short, long, group = "req")]
    pub require: Option<String>,

    /// Modify '-r, --require' to require matching on all specified capture names or any.
    #[arg(long, requires = "req", default_value_t = RequireMode::default())]
    pub require_mode: RequireMode,

    /// Force output to be line-buffered. By default, output is line buffered when stdout is a
    /// terminal and block-buffered otherwise.
    #[arg(long)]
    pub line_buffered: bool,

    /// Produce completions for shell and exit.
    #[arg(short, long)]
    pub completions: Option<clap_complete::Shell>,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum RequireMode {
    /// Require all capture names specified to be matched for input line to be processed.
    #[default]
    All,
    /// Require at least one capture name among those specified to be matched for input line to be
    /// processed.
    Any,
}

impl fmt::Display for RequireMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::Any => write!(f, "any"),
        }
    }
}

impl Cli {
    pub fn compute_shell_used_for_completions() -> Result<Option<Shell>> {
        let mut raw_args = env::args_os();

        if raw_args.any(|a| {
            a.as_os_str()
                .to_str()
                .is_some_and(|s| s == "--completions" || s == "-c")
        }) {
            if let Some(raw_shell) = raw_args.next().and_then(|a| a.as_os_str().to_str().map(String::from)) {
                let shell = <Shell as FromStr>::from_str(&raw_shell.to_lowercase())
                    .map_err(|e| format_err!("failed to determine which Shell to generate autocomplete due to {e}"))?;
                return Ok(Some(shell));
            }
        }
        Ok(None)
    }
}
