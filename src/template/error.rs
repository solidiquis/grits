use super::{
    rules::VALID_ANCHOR_NAME,
    token::{ANCHOR, ANCHOR_CLOSE, ANCHOR_OPEN, ESCAPE},
};
use indoc::{formatdoc, indoc};
use std::fmt::{self, Display};

#[derive(Debug)]
pub struct ParseError {
    message: String,
    // Partial or full template string to include in output message
    partial_template: String,
    // The index of the beginning char that caused the error
    char_index: usize,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ParseError {
            message,
            partial_template,
            char_index,
        } = self;

        let mut error_position_display = [' '].repeat(partial_template.len());
        error_position_display[*char_index] = '^';

        let error_position: String = error_position_display.into_iter().collect();

        let output = formatdoc! {"
            Something went wrong while parsing the provided output template:
                {partial_template}
                {error_position}
            {message}
        "};
        write!(f, "{output}")
    }
}

impl std::error::Error for ParseError {}

impl ParseError {
    pub fn missing_escapee(char_index: usize, chars: &[char]) -> Self {
        ParseError {
            char_index,
            partial_template: chars.iter().collect(),
            message: format!(
                "A character immediately following the '{ESCAPE}' escape is required."
            ),
        }
    }

    pub fn invalid_anchor_start(char_index: usize, chars: &[char]) -> Self {
        ParseError {
            char_index,
            partial_template: chars.iter().collect(),
            message: format!("Expected '{ANCHOR_OPEN}' to immediately follow '{ANCHOR}'."),
        }
    }

    pub fn invalid_anchor_name(char_index: usize, chars: &[char]) -> Self {
        ParseError {
            char_index,
            partial_template: chars.iter().collect(),
            message: format!("Anchor name cannot be blank and must satisfy the following regular expression: {VALID_ANCHOR_NAME}"),
        }
    }

    pub fn unclosed_anchor(char_index: usize, chars: &[char]) -> Self {
        ParseError {
            char_index,
            partial_template: chars.iter().collect(),
            message: format!("Expected a '{ANCHOR_CLOSE}' character to close anchor declaration."),
        }
    }

    pub fn invalid_index(char_index: usize, chars: &[char]) -> Self {
        ParseError {
            char_index,
            partial_template: chars.iter().collect(),
            message: String::from("Expected index to be numeric."),
        }
    }

    pub fn invalid_indexing_operation(char_index: usize, chars: &[char]) -> Self {
        ParseError {
            char_index,
            partial_template: chars.iter().collect(),
            message: indoc! {"
                Invalid index operation. Example of a valid index operation:
                    - '${foo[0]}'
            "}
            .to_string(),
        }
    }

    pub fn invalid_default_value_operation(char_index: usize, chars: &[char]) -> Self {
        ParseError {
            char_index,
            partial_template: chars.iter().collect(),
            message: indoc! {"
                Invalid default value operation. Examples of a valid index operations:
                    - Default to string literal: '${foo || \"bar\"}'
                    - Default to another anchor: '${foo || bar}'
                    - Default to another indexed-anchor: '${foo || bar[0]}'
                    - Chaining defaults: '${foo || bar || baz}'
            "}
            .to_string(),
        }
    }

    pub fn default_str_literal_missing_closing_quote(char_index: usize, chars: &[char]) -> Self {
        ParseError {
            char_index,
            partial_template: chars.iter().collect(),
            message: String::from("Default string literal missing closing quote."),
        }
    }

    pub fn default_parsing_disallowed_char(char_index: usize, chars: &[char]) -> Self {
        ParseError {
            char_index,
            partial_template: chars.iter().collect(),
            message: String::from("Invalid syntax: encountered a disallowed character while parsing default value operation.")
        }
    }

    pub fn index_parsing_eol(char_index: usize, chars: &[char]) -> Self {
        ParseError {
            char_index,
            partial_template: chars.iter().collect(),
            message: String::from("Invalid syntax: template string ends prematurely while parsing an index operation.")
        }
    }

    pub fn default_parsing_eol(char_index: usize, chars: &[char]) -> Self {
        ParseError {
            char_index,
            partial_template: chars.iter().collect(),
            message: String::from(
                "Invalid syntax: template string ends prematurely while parsing default values.",
            ),
        }
    }
}

#[test]
fn test_parse_error_display() {
    use indoc::indoc;

    let template: Vec<char> = "output=${foo} \\".chars().collect();
    let error = ParseError::missing_escapee(14, &template);

    assert_eq!(
        format!("{error}"),
        indoc! {"
            Something went wrong while parsing the provided output template:
                output=${foo} \\
                              ^
            A character immediately following the '\\' escape is required.
        "},
    )
}
