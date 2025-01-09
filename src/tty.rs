use anyhow::Result;
use crossterm::style::Stylize;
use std::{
    fmt::Display,
    io::{stderr, stdout, IsTerminal, Stderr, Stdout, Write},
};

#[derive(Debug)]
pub struct TtyContext {
    pub stdout_is_terminal: bool,
    pub stdout: Stdout,
    pub stderr: Stderr,
}

impl Default for TtyContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TtyContext {
    pub fn new() -> Self {
        let enable_color = std::env::var("NO_COLOR").unwrap_or_default().is_empty();
        log::debug!("color enabled: {enable_color}");
        crossterm::style::force_color_output(enable_color);

        let stdout = stdout();
        let stdout_is_terminal = stdout.is_terminal();
        log::debug!("stdout is terminal: {stdout_is_terminal}");

        let stderr = stderr();

        Self {
            stderr,
            stdout,
            stdout_is_terminal,
        }
    }

    pub fn writeln<T: Display>(&mut self, content: T) -> Result<()> {
        writeln!(self.stdout, "{}", content)?;
        Ok(())
    }

    pub fn write_err(&mut self, err: &str) -> Result<()> {
        writeln!(self.stderr, "{}", err)?;
        Ok(())
    }
}
