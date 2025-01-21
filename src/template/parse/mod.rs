use super::{
    error::ParseError,
    token::{
        ANCHOR_CLOSE, ANCHOR_OPEN, ATTRIBUTE_CLOSE, ATTRIBUTE_DELIMETER, ATTRIBUTE_END, ATTRIBUTE_OPEN, DEFAULT_PIPE,
        ESCAPE, INDEX_CLOSE, INDEX_OPEN, LITERAL_DOUBLE_QUOTE, LITERAL_SINGLE_QUOTE, PARAM_CLOSE, PARAM_OPEN, REQUIRED,
    },
};
use anyhow::{format_err, Result};
use std::{
    cmp::Ordering,
    fmt::{self, Debug},
};

/// Concerned with ANSI-escape sequences that can be applied to anchors.
pub mod attr;
pub use attr::{Attribute, AttributeKind};

/// Concerned with validating certain properties that are computed during parsing
/// such as anchor name.
pub mod rules;
use rules::Rules;

#[cfg(test)]
pub mod test;

/// An interpolation point with additional properties that affect how text is transformed.
/// The `name` field should be identical with the regular expression capture group whose value
/// will be used to replace the anchor. The `start` and `end` field mark the range in which the
/// anchor appears in the original template string. The `index` determines which value amongst the
/// captures will be used for interpolation (defaults to 0). The `defaults` field contains
/// fallbacks in case an anchor doesn't have an associated match. The first non-blank value amongst
/// the defaults will be used for interpolation. The `attributes` fields applies ANSI-escape
/// sequences to the interpolated value.
///
/// If `name` is empty then default is expected to contain a single literal value.
#[derive(Debug, Default, Clone)]
pub struct Anchor {
    pub name: String,
    pub start: usize,
    pub end: usize,
    pub index: Option<usize>,
    pub defaults: Vec<DefaultValue>,
    pub attributes: Vec<Attribute>,
    pub required: bool,
}

/// State that is maintained during parsing. The `cursor` is the index of the current token
/// that we are on amongst `tokens`. The `mode` field determines which phase we are in during
/// parsing. The `bound_anchor` is the anchor that we are currently working on.
struct ParseState {
    cursor: usize,
    tokens: Vec<char>,
    mode: ParseStateMode,
    bound_anchor: Option<Anchor>,
    /// For debugging purposes only
    recursion_depth: usize,
}

/// Determines which mode of parsing we are in.
#[derive(Debug)]
enum ParseStateMode {
    /// Walk through regular characters.
    Base,
    /// Encountered an escape character which will cause the next token to be treated as a
    /// non-special character.
    Escaping,
    /// Encountered `{` which begins the anchor.
    AnchorBegin,
    /// Parse anchor name
    AnchorParseBase,
    /// Encountered a `[` which indicates indexing operation.
    AnchorParseIndex,
    /// Encountered two '|' in succession which while anchor parsing which indicates default value.
    AnchorParseDefaultValue,
    /// Encountered a '"' or a '\'' opening quote which indicates a default literal value.
    AnchorParseDefaultLiteral,
    /// User is using an anchor as a default value.
    AnchorParseDefaultAnchor,
    /// Encountered a '(' while parsing an achor indicating attribute usage
    AttributeParse,
}

#[derive(Debug, Clone)]
pub enum DefaultValue {
    Literal(String),
    /// Unlike a regular anchor, this one is unconcerned about position
    Anchor {
        name: String,
        index: Option<usize>,
    },
}

/// Parses the user-sourced template string.
pub(super) fn parse(template: &str) -> Result<Vec<Anchor>> {
    let mut mode = ParseState {
        cursor: 0,
        tokens: template.chars().collect(),
        mode: ParseStateMode::Base,
        bound_anchor: None,
        recursion_depth: 0,
    };
    let rules = Rules::new();
    let mut anchors = Vec::new();
    parse_impl(&mut mode, &mut anchors, &rules)?;

    Ok(anchors)
}

/// Finite mode machine
fn parse_impl(mode: &mut ParseState, anchors: &mut Vec<Anchor>, rules: &Rules) -> Result<()> {
    mode.recursion_depth += 1;
    log::debug!("{mode:?}");

    match mode.mode {
        // Contains the base-case
        ParseStateMode::Base => {
            for i in mode.cursor..mode.tokens.len() {
                mode.cursor = i;
                let Some(token) = mode.tokens.get(i).copied() else {
                    return Ok(());
                };
                if token == ESCAPE {
                    mode.mode = ParseStateMode::Escaping;
                    return parse_impl(mode, anchors, rules);
                } else if token == ANCHOR_OPEN {
                    mode.mode = ParseStateMode::AnchorBegin;
                    mode.bound_anchor = Some(Anchor {
                        start: mode.cursor,
                        ..Default::default()
                    });
                    return parse_impl(mode, anchors, rules);
                }
            }
            Ok(())
        }

        ParseStateMode::Escaping => {
            mode.cursor += 1;
            if mode.tokens.get(mode.cursor).is_none() {
                return Err(ParseError::missing_escapee(mode.cursor - 1, &mode.tokens).into());
            }
            mode.cursor += 1;
            mode.mode = ParseStateMode::Base;
            parse_impl(mode, anchors, rules)
        }

        ParseStateMode::AnchorBegin => {
            mode.cursor += 1;
            for i in mode.cursor..mode.tokens.len() {
                if mode
                    .tokens
                    .get(mode.cursor)
                    .is_some_and(|token| token.is_ascii_whitespace())
                {
                    mode.cursor = i;
                    break;
                }
                continue;
            }
            if mode.tokens.get(mode.cursor).is_some_and(|token| *token == REQUIRED) {
                let Some(anchor) = mode.bound_anchor.as_mut() else {
                    log::error!("expected mode.bound_anchor to be `Some` while in `AnchorParseBegin`");
                    return Err(format_err!(
                        "An unexpected error occurred while parsing template string."
                    ));
                };
                anchor.required = true;
                mode.cursor += 1;
            }
            if mode
                .tokens
                .get(mode.cursor)
                .is_some_and(|token| *token == ATTRIBUTE_OPEN)
            {
                mode.mode = ParseStateMode::AttributeParse;
            } else {
                mode.mode = ParseStateMode::AnchorParseBase;
            }
            parse_impl(mode, anchors, rules)
        }

        ParseStateMode::AnchorParseBase => {
            let begin = mode.cursor;
            let mut end = mode.cursor;

            let Some(anchor) = mode.bound_anchor.as_mut() else {
                log::error!("expected mode.bound_anchor to be `Some` while in `AnchorParseBase`");
                return Err(format_err!(
                    "An unexpected error occurred while parsing template string."
                ));
            };

            for i in mode.cursor..mode.tokens.len() {
                mode.cursor = i;
                let Some(token) = mode.tokens.get(mode.cursor).copied() else {
                    return Err(ParseError::unclosed_anchor(mode.cursor - 1, &mode.tokens).into());
                };
                if token == INDEX_OPEN {
                    for token in &mode.tokens[begin..end] {
                        if token.is_ascii_whitespace() {
                            continue;
                        }
                        anchor.name.push(*token)
                    }
                    let prev_token = mode.tokens.get(mode.cursor - 1).copied();

                    if anchor.name.is_empty() || anchor.name.chars().last().is_some_and(|c| Some(c) != prev_token) {
                        return Err(ParseError::invalid_indexing_operation(mode.cursor - 1, &mode.tokens).into());
                    }
                    mode.mode = ParseStateMode::AnchorParseIndex;
                    break;
                } else if token == ANCHOR_CLOSE {
                    for token in &mode.tokens[begin..end] {
                        if token.is_ascii_whitespace() {
                            continue;
                        }
                        anchor.name.push(*token)
                    }
                    if (anchor.name.is_empty() || !rules.name_is_valid(&anchor.name)) && anchor.defaults.is_empty() {
                        return Err(ParseError::invalid_anchor_name(mode.cursor - 1, &mode.tokens).into());
                    }
                    mode.mode = ParseStateMode::Base;

                    let Some(mut anchor) = mode.bound_anchor.take() else {
                        log::error!("expected mode.bound_anchor to be `Some` just before completing `AnchorParseBase`");
                        return Err(format_err!(
                            "An unexpected error occurred while parsing template string."
                        ));
                    };
                    anchor.end = mode.cursor + 1;
                    anchors.push(anchor);
                    break;
                } else if token == DEFAULT_PIPE {
                    for token in &mode.tokens[begin..end] {
                        if token.is_ascii_whitespace() {
                            continue;
                        }
                        anchor.name.push(*token)
                    }
                    mode.cursor += 1;
                    let next_token_is_pipe = mode.tokens.get(mode.cursor).is_some_and(|token| *token == DEFAULT_PIPE);

                    if anchor.name.is_empty() || !next_token_is_pipe {
                        return Err(ParseError::invalid_default_value_operation(mode.cursor - 1, &mode.tokens).into());
                    }
                    mode.mode = ParseStateMode::AnchorParseDefaultValue;
                    break;
                } else if token == LITERAL_DOUBLE_QUOTE || token == LITERAL_SINGLE_QUOTE {
                    mode.mode = ParseStateMode::AnchorParseDefaultLiteral;
                    break;
                }
                end += 1;
            }
            parse_impl(mode, anchors, rules)
        }

        ParseStateMode::AnchorParseIndex => {
            mode.cursor += 1;
            let begin = mode.cursor;
            let mut end = begin;

            let Some(anchor) = mode.bound_anchor.as_mut() else {
                log::error!("expected mode.bound_anchor to be `Some` during `AnchorParseIndex`");
                return Err(format_err!(
                    "An unexpected error occurred while parsing template string."
                ));
            };
            for i in mode.cursor..mode.tokens.len() {
                mode.cursor = i;
                if mode.tokens.get(i).is_some_and(|token| *token == INDEX_CLOSE) {
                    mode.cursor += 1;
                    break;
                }
                end += 1;
            }
            let index = mode.tokens[begin..end]
                .iter()
                .collect::<String>()
                .parse::<usize>()
                .map_err(|_| ParseError::invalid_index(mode.cursor - 1, &mode.tokens))?;
            anchor.index = Some(index);

            mode.mode = ParseStateMode::AnchorParseBase;
            parse_impl(mode, anchors, rules)
        }

        ParseStateMode::AnchorParseDefaultValue => {
            if mode.bound_anchor.as_ref().is_some_and(|a| a.required) {
                return Err(ParseError::default_disallowed_with_required(mode.cursor - 1, &mode.tokens).into());
            }

            mode.cursor += 1;
            for i in mode.cursor..mode.tokens.len() {
                mode.cursor = i;

                let Some(token) = mode.tokens.get(mode.cursor).copied() else {
                    break;
                };
                if token.is_ascii_whitespace() {
                    continue;
                } else if token == LITERAL_SINGLE_QUOTE || token == LITERAL_DOUBLE_QUOTE {
                    mode.cursor -= 1;
                    mode.mode = ParseStateMode::AnchorParseDefaultLiteral;
                    return parse_impl(mode, anchors, rules);
                } else if rules.name_is_valid(&token.to_string()) {
                    mode.cursor -= 1;
                    mode.mode = ParseStateMode::AnchorParseDefaultAnchor;
                    return parse_impl(mode, anchors, rules);
                } else {
                    return Err(ParseError::default_parsing_disallowed_char(mode.cursor - 1, &mode.tokens).into());
                }
            }
            Err(ParseError::default_parsing_eol(mode.cursor - 1, &mode.tokens).into())
        }

        ParseStateMode::AnchorParseDefaultLiteral => {
            for i in mode.cursor..mode.tokens.len() {
                mode.cursor = i;
                if mode
                    .tokens
                    .get(i)
                    .copied()
                    .is_some_and(|token| token.is_ascii_whitespace())
                {
                    continue;
                }
                break;
            }
            let Some(opening_quote) = mode.tokens.get(mode.cursor).copied() else {
                log::error!("expected opening quote that starts the default literal in `AnchorParseDefaultLiteral`");
                return Err(format_err!(
                    "An unexpected error occurred while parsing template string."
                ));
            };
            mode.cursor += 1;

            let begin = mode.cursor;
            let mut end = mode.cursor + 1;

            for _ in mode.cursor..mode.tokens.len() {
                mode.cursor += 1;

                let Some(token) = mode.tokens.get(mode.cursor).copied() else {
                    break;
                };
                if token == ESCAPE {
                    mode.cursor += 1;
                    continue;
                } else if token == opening_quote {
                    let Some(bound_anchor) = mode.bound_anchor.as_mut() else {
                        log::error!("expected mode.bound_anchor to be `Some` during `AnchorParseDefaultLiteral`");
                        return Err(format_err!(
                            "An unexpected error occurred while parsing template string."
                        ));
                    };
                    let literal_value = mode.tokens[begin..end].iter().collect::<String>();
                    bound_anchor.defaults.push(DefaultValue::Literal(literal_value));
                    mode.mode = ParseStateMode::AnchorParseBase;
                    mode.cursor += 1;
                    return parse_impl(mode, anchors, rules);
                }
                end += 1;
            }
            Err(ParseError::default_str_literal_missing_closing_quote(mode.cursor - 1, &mode.tokens).into())
        }

        ParseStateMode::AnchorParseDefaultAnchor => {
            mode.cursor += 1;
            for i in mode.cursor..mode.tokens.len() {
                mode.cursor = i;
                if mode
                    .tokens
                    .get(i)
                    .copied()
                    .is_some_and(|token| token.is_ascii_whitespace())
                {
                    continue;
                }
                break;
            }
            let begin = mode.cursor;
            let mut end = begin + 1;
            let mut index = None;

            while mode.cursor < mode.tokens.len() {
                mode.cursor += 1;

                let Some(token) = mode.tokens.get(mode.cursor).copied() else {
                    break;
                };
                if index.is_some() || token.is_ascii_whitespace() || token == DEFAULT_PIPE || token == ANCHOR_CLOSE {
                    let name: String = mode.tokens[begin..end].iter().collect();
                    if !rules.name_is_valid(&name) {
                        return Err(ParseError::invalid_anchor_name(mode.cursor - 1, &mode.tokens).into());
                    }
                    let Some(anchor) = mode.bound_anchor.as_mut() else {
                        log::error!("expected mode.bound_anchor to be `Some` while in `AnchorParseDefaultAnchor`");
                        return Err(format_err!(
                            "An unexpected error occurred while parsing template string."
                        ));
                    };
                    anchor.defaults.push(DefaultValue::Anchor { name, index });
                    mode.mode = ParseStateMode::AnchorParseBase;
                    return parse_impl(mode, anchors, rules);
                } else if token == INDEX_OPEN {
                    mode.cursor += 1;
                    let index_begin = mode.cursor;
                    let mut index_end = index_begin + 1;

                    for _ in mode.cursor..mode.tokens.len() {
                        mode.cursor += 1;
                        let Some(token) = mode.tokens.get(mode.cursor).copied() else {
                            return Err(ParseError::index_parsing_eol(mode.cursor - 1, &mode.tokens).into());
                        };
                        if token == INDEX_CLOSE {
                            break;
                        }
                        index_end += 1;
                    }
                    let parsed_index = mode.tokens[index_begin..index_end]
                        .iter()
                        .collect::<String>()
                        .parse::<usize>()
                        .map_err(|_| ParseError::invalid_index(mode.cursor - 1, &mode.tokens))?;
                    index = Some(parsed_index);
                    continue;
                }
                end += 1;
            }
            Err(ParseError::default_parsing_eol(mode.cursor - 1, &mode.tokens).into())
        }

        ParseStateMode::AttributeParse => {
            mode.cursor += 1;
            let mut start = mode.cursor;
            let mut closed = false;

            let mut raw_attrs: Vec<(String, Option<String>)> = Vec::new();

            while mode.cursor < mode.tokens.len() {
                mode.cursor += 1;
                let Some(token) = mode.tokens.get(mode.cursor).copied() else {
                    continue;
                };
                if token == ATTRIBUTE_CLOSE {
                    if mode.cursor - start > 1 {
                        let attr = mode.tokens[start..mode.cursor].iter().collect::<String>();
                        raw_attrs.push((attr, None));
                    }
                    closed = true;
                    break;
                } else if token == ATTRIBUTE_DELIMETER || token == PARAM_OPEN {
                    if mode.cursor - start <= 1 {
                        start = mode.cursor + 1;
                        continue;
                    }
                    let attr = mode.tokens[start..mode.cursor].iter().collect::<String>();
                    let mut args = None;
                    start = mode.cursor + 1;

                    if token == PARAM_OPEN {
                        mode.cursor += 1;

                        let params_start = mode.cursor;
                        let mut params_end = mode.cursor;
                        let mut current_token = mode.tokens.get(mode.cursor).copied();
                        let mut open_quote: Option<char> = None;

                        while let Some(token) = current_token {
                            if token == PARAM_CLOSE && open_quote.is_none() {
                                start = mode.cursor;
                                break;
                            } else if token == ESCAPE {
                                mode.cursor += 1
                            } else if token == LITERAL_SINGLE_QUOTE || token == LITERAL_DOUBLE_QUOTE {
                                if let Some(quote) = open_quote {
                                    if token != quote {
                                        return Err(ParseError::string_parameter_missing_closing_quote(
                                            start,
                                            &mode.tokens,
                                        )
                                        .into());
                                    }
                                    open_quote = None;
                                } else {
                                    open_quote = Some(token);
                                }
                            }
                            mode.cursor += 1;
                            params_end += 1;
                            current_token = mode.tokens.get(mode.cursor).copied();
                        }
                        args = Some(mode.tokens[params_start..params_end].iter().collect::<String>());
                    }
                    raw_attrs.push((attr, args));
                }
            }
            if !closed {
                return Err(ParseError::attribute_unclosed(start, &mode.tokens).into());
            }
            mode.cursor += 1;

            if mode.tokens.get(mode.cursor).is_none_or(|token| *token != ATTRIBUTE_END) {
                return Err(ParseError::attribute_end(mode.cursor - 1, &mode.tokens).into());
            }

            let mut attrs = Vec::with_capacity(raw_attrs.len());
            for (name, args) in raw_attrs {
                let attr = Attribute::parse(name, args)?;
                attrs.push(attr);
            }

            // Alignment always comes first due to ANSI-escape sequences messing with string length
            attrs.sort_by(|a, b| {
                if let AttributeKind::Align { .. } = a.kind {
                    Ordering::Less
                } else if let AttributeKind::Align { .. } = b.kind {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            });

            if let Some(anchor) = mode.bound_anchor.as_mut() {
                anchor.attributes = attrs;
            } else {
                log::error!("expected mode.bound_anchor to be `Some` while in `AttributeParse`");
                return Err(format_err!(
                    "An unexpected error occurred while parsing template string."
                ));
            }
            mode.cursor += 1;
            mode.mode = ParseStateMode::AnchorParseBase;
            parse_impl(mode, anchors, rules)
        }
    }
}

impl Debug for ParseState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ParseState {
            recursion_depth,
            cursor,
            mode,
            bound_anchor,
            ..
        } = self;
        write!(
            f,
            "ParseState: depth={recursion_depth} cursor={cursor} mode={mode:?}, bound_anchor={bound_anchor:?}"
        )
    }
}
