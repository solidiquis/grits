use anyhow::Result;

mod rules;
mod token;

pub mod parse;
pub use parse::Anchor;

pub mod error;

pub struct OutputTemplate {
    template: String,
    anchors: Vec<Anchor>,
}

impl OutputTemplate {
    fn parse() -> Result<Self> {
        todo!()
    }
}
