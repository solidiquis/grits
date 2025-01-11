use anyhow::{Context, Result};
use std::{
    io::{stderr, stdout, IsTerminal, Stderr, Stdout, StdoutLock, Write},
    ops::Drop,
};

/// Traditional block size in bytes
const BLOCK_SIZE: usize = 512;

/// Entry-point for the rest of the application to access handlers
/// to stdout and stderr. Also enables/disable colorization for the
/// application based on whether stdout is a tty and if the `NO_COLOR`
/// environment variable is set. For there to be colorzation stdout must
/// be a tty and `NO_COLOR` must be blank.
#[derive(Debug)]
pub struct TtyContext {
    pub stdout: Stdout,
    pub stderr: Stderr,
}

/// Contains behavior to write to output.
pub trait OutputWriter {
    fn writeln(&mut self, txt: &str) -> Result<()>;
}

/// Writes directly to stdout in a line-buffered manner.
pub struct LineBufferedOutputWriter<'a> {
    stdout_lock: StdoutLock<'a>,
}

/// Writes to stdout in a block-buffered manner. Any contents that remain in the buffer that
/// weren't manually flushed will be flushed when dropped.
pub struct BlockBufferedOutputWriter<'a> {
    stdout_lock: StdoutLock<'a>,
    buffer: Vec<u8>,
}

impl Default for TtyContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns a [LineBufferedOutputWriter] if stdout is a terminal or if `line_buffered` is
/// `true`, otherwise returns a [BlockBufferedOutputWriter].
pub fn init_output_writer(tty: &TtyContext, line_buffered: bool) -> Box<dyn OutputWriter> {
    let stdout = tty.stdout.lock();

    if tty.stdout.is_terminal() || line_buffered {
        log::debug!("line buffered");
        return Box::new(LineBufferedOutputWriter::new(stdout));
    }
    log::debug!("block buffered");
    Box::new(BlockBufferedOutputWriter::new(stdout))
}

impl TtyContext {
    pub fn new() -> Self {
        let stdout = stdout();
        let stdout_is_terminal = stdout.is_terminal();
        log::debug!("stdout is terminal: {stdout_is_terminal}");

        let enable_color = stdout_is_terminal && std::env::var("NO_COLOR").unwrap_or_default().is_empty();
        log::debug!("color enabled: {enable_color}");
        crossterm::style::force_color_output(enable_color);

        let stderr = stderr();

        Self { stderr, stdout }
    }

    pub fn write_err(&mut self, err: &str) -> Result<()> {
        writeln!(self.stderr, "{}", err)?;
        Ok(())
    }
}

impl<'a> LineBufferedOutputWriter<'a> {
    pub fn new(stdout_lock: StdoutLock<'a>) -> Self {
        Self { stdout_lock }
    }
}

impl<'a> BlockBufferedOutputWriter<'a> {
    pub fn new(stdout_lock: StdoutLock<'a>) -> Self {
        Self {
            stdout_lock,
            buffer: Vec::with_capacity(BLOCK_SIZE),
        }
    }

    /// Flushes and clears the buffer.
    fn flush_buffer(&mut self) -> Result<()> {
        if !self.buffer.is_empty() {
            self.stdout_lock
                .write_all(&self.buffer)
                .context("failed to write buffer to stdout")?;
            self.buffer.clear();
        }
        Ok(())
    }
}

impl OutputWriter for LineBufferedOutputWriter<'_> {
    fn writeln(&mut self, txt: &str) -> Result<()> {
        writeln!(&mut self.stdout_lock, "{txt}").context("something went wrong while trying to write to stdout")?;
        Ok(())
    }
}

impl OutputWriter for BlockBufferedOutputWriter<'_> {
    fn writeln(&mut self, txt: &str) -> Result<()> {
        let txt_bytes = txt.as_bytes();

        if self.buffer.len() + txt_bytes.len() + 1 > BLOCK_SIZE {
            self.flush_buffer()?;
        }
        self.buffer.extend_from_slice(txt_bytes);
        self.buffer.push(b'\n');

        if self.buffer.len() >= BLOCK_SIZE {
            self.flush_buffer()?;
        }
        Ok(())
    }
}

impl Drop for BlockBufferedOutputWriter<'_> {
    fn drop(&mut self) {
        if let Err(e) = self.flush_buffer() {
            log::warn!(
                "failed to flush buffer of block buffered output writer before dropping: {}",
                e
            );
        }
    }
}
