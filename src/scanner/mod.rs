/// Concerned with reading input lines from multiple file sources.
pub mod file;
pub use file::MultiFileScanner;

/// Concerned with reading input lines from standard input.
pub mod stdin;
pub use stdin::StdinScanner;
