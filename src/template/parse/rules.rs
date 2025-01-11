use super::super::token::ATTRIBUTE_DELIMETER;
use regex::Regex;

/// Defines a valid anchor name
pub const VALID_ANCHOR_CHARSET: &str = r#"^[a-zA-Z0-9_]+$"#;

/// Concerned with enforcing validations for various properties
/// computed during parsing such as anchor name.
pub struct Rules {
    valid_anchor_name: Regex,
}

impl Rules {
    pub fn new() -> Self {
        Self {
            valid_anchor_name: Regex::new(VALID_ANCHOR_CHARSET).unwrap(),
        }
    }

    /// Is the anchor name valid
    pub fn name_is_valid(&self, name: &str) -> bool {
        self.valid_anchor_name.is_match(name)
    }

    /// Determines if the attribute string i.e. `(red|bold)` in `${(red|bold):foo}`
    /// is valid during parsing.
    pub fn in_attr_string_charset(&self, ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == ATTRIBUTE_DELIMETER
    }
}
