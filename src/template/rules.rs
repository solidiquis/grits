use regex::Regex;

pub const VALID_ANCHOR_NAME: &str = "[a-zA-Z0-9_]";

pub struct Rules {
    valid_anchor_name: Regex,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            valid_anchor_name: Regex::new(VALID_ANCHOR_NAME).unwrap(),
        }
    }
}
