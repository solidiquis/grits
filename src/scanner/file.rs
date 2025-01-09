use anyhow::{format_err, Context, Result};
use std::{
    fs::File,
    io::{BufRead, BufReader, Lines},
    path::Path,
};

pub struct MultiFileScanner {
    current_buf_reader_idx: usize,
    buf_readers: Vec<Lines<BufReader<File>>>,
}

impl MultiFileScanner {
    fn new<F: AsRef<Path>>(file_paths: &[F]) -> Result<Self> {
        if file_paths.is_empty() {
            return Err(format_err!(
                "MultiFileScanner cannot be created without input files"
            ));
        }
        let mut buf_readers = Vec::with_capacity(file_paths.len());

        for file_path in file_paths {
            let buf_reader = File::open(file_path)
                .map(BufReader::new)
                .context("failed to open an input file")?;
            buf_readers.push(buf_reader.lines());
        }
        let current_buf_reader_idx = usize::default();

        Ok(MultiFileScanner {
            current_buf_reader_idx,
            buf_readers,
        })
    }

    pub fn init<F: AsRef<Path>>(file_paths: &[F]) -> Result<Box<dyn Iterator<Item = String>>> {
        let scanner = Self::new(file_paths).map(Box::new)?;
        Ok(scanner)
    }
}

impl Iterator for MultiFileScanner {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let buf_reader = self.buf_readers.get_mut(self.current_buf_reader_idx)?;
        match buf_reader.next() {
            Some(Ok(line)) => Some(line),
            _ => {
                self.current_buf_reader_idx += 1;
                self.next()
            }
        }
    }
}
