use anyhow::Result;
use std::collections::HashMap;

/// Tokens with special meaning used in the template string
mod token;

pub mod parse;
pub use parse::{Anchor, Attribute, DefaultValue};

#[cfg(test)]
mod test;

/// Errors specific to template string parsing
pub mod error;

/// The actual template concerned with generating the output string.
#[derive(Default, Debug)]
pub struct OutputTemplate {
    targets: Vec<InterpolationTarget>,
}

/// Utility type that defines a segment of the output which is defined either by a literal or an
/// anchor.
#[derive(Debug)]
enum InterpolationTarget {
    Literal(String),
    Anchor(Anchor),
}

impl OutputTemplate {
    /// Parses the input `template` string to create the output template that is
    /// ready to produce an output.
    pub fn parse(template: &str) -> Result<Self> {
        let anchors = parse::parse(template)?;

        let mut targets = Vec::new();

        let mut left_cursor = 0;
        let mut right_cursor = 0;

        for anchor in &anchors {
            let start = left_cursor;
            for i in start..template.len() {
                right_cursor = i;
                if right_cursor == anchor.start {
                    if left_cursor != right_cursor {
                        let section = String::from(&template[left_cursor..right_cursor]);
                        targets.push(InterpolationTarget::Literal(section));
                    }
                    targets.push(InterpolationTarget::Anchor(anchor.clone()));

                    left_cursor = anchor.end;
                    right_cursor = left_cursor;
                }
            }
        }
        if right_cursor != template.len() {
            let section = String::from(&template[left_cursor..]);
            if !section.is_empty() {
                targets.push(InterpolationTarget::Literal(section));
            }
        }
        Ok(Self { targets })
    }

    /// The actual transformation logic. The original template string that is provided
    /// is used in conjunction with the `interpolation_map` to produce the transformed
    /// output. The key of the map is the name of anchor while the associated value is
    /// a vector containing the possible values used to interpolate the anchor-sites
    /// in the template string, depending on whether an index is specified.
    pub fn transform(&self, interpolation_map: &HashMap<&str, Vec<&str>>) -> String {
        let mut out = String::new();

        for target in &self.targets {
            match target {
                InterpolationTarget::Anchor(anchor) => {
                    let name = anchor.name.as_str();
                    let index = anchor.index.unwrap_or_default();

                    if let Some(val) = interpolation_map.get(name).and_then(|vals| vals.get(index)) {
                        if anchor.attributes.is_empty() {
                            out.push_str(val);
                        } else {
                            let stylized = Attribute::apply(val, &anchor.attributes);
                            out.push_str(&stylized);
                        }
                        continue;
                    }

                    // No match, return empty string.
                    if anchor.required {
                        return String::new();
                    }

                    for default_val in &anchor.defaults {
                        match default_val {
                            DefaultValue::Literal(val) => {
                                if anchor.attributes.is_empty() {
                                    out.push_str(val);
                                } else {
                                    let stylized = Attribute::apply(val, &anchor.attributes);
                                    out.push_str(&stylized);
                                }
                                break;
                            }
                            DefaultValue::Anchor { name, index } => {
                                let name = name.as_str();
                                let index = index.unwrap_or_default();
                                if let Some(val) = interpolation_map.get(name).and_then(|vals| vals.get(index)) {
                                    if anchor.attributes.is_empty() {
                                        out.push_str(val);
                                    } else {
                                        let stylized = Attribute::apply(val, &anchor.attributes);
                                        out.push_str(&stylized);
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
                InterpolationTarget::Literal(val) => out.push_str(val),
            }
        }
        out
    }
}
