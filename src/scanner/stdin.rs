use std::io::{self, Lines, StdinLock};

/// A type that implements [Iterator] to iterate through lines from standard input.
pub struct StdinScanner {
    inner: Lines<StdinLock<'static>>,
}

impl StdinScanner {
    fn new() -> Self {
        let inner = io::stdin().lines();
        Self { inner }
    }

    pub fn init() -> Box<dyn Iterator<Item = String>> {
        Box::new(Self::new())
    }
}

impl Iterator for StdinScanner {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(Ok(val)) => Some(val),
            _ => None,
        }
    }
}
