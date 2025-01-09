use super::{
    error::ParseError,
    token::{
        ANCHOR, ANCHOR_CLOSE, ANCHOR_OPEN, ATTRIBUTE_CLOSE, ATTRIBUTE_OPEN, DEFAULT_PIPE, ESCAPE,
        INDEX_CLOSE, INDEX_OPEN, LITERAL_DOUBLE_QUOTE, LITERAL_SINGLE_QUOTE,
    },
};
use anyhow::{format_err, Result};
use regex::Regex;
use std::fmt::{self, Debug};

#[cfg(test)]
pub mod test;

#[derive(Debug, Default)]
pub struct Anchor {
    pub name: String,
    pub start: usize,
    pub end: usize,
    pub index: Option<usize>,
    /// Users can chain defaults - when interpolation occurs the first default value that
    /// succesfuly produces a match will be used.
    pub defaults: Vec<DefaultValue>,
}

struct ParseState {
    cursor: usize,
    tokens: Vec<char>,
    state: ParseStateMachine,
    bound_anchor: Option<Anchor>,
    recursion_depth: usize,
}

#[derive(Debug)]
enum ParseStateMachine {
    /// Walk through regular characters.
    Base,
    /// Encountered an escape character which will cause the next token to be treated as a
    /// non-special character.
    Escaping,
    /// Encountered `$` which begins the anchor.
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

#[derive(Debug)]
pub enum DefaultValue {
    Literal(String),
    /// Unlike a regular anchor, this one is unconcerned about position
    Anchor {
        name: String,
        index: Option<usize>,
    },
}

pub struct Rules {
    valid_anchor_name: Regex,
}

pub fn parse(template: &str) -> Result<Vec<Anchor>> {
    let mut state = ParseState {
        cursor: 0,
        tokens: template.chars().collect(),
        state: ParseStateMachine::Base,
        bound_anchor: None,
        recursion_depth: 0,
    };

    let rules = Rules {
        valid_anchor_name: Regex::new(r"^[a-zA-Z0-9_]+$").unwrap(),
    };

    let mut anchors = Vec::new();
    parse_impl(&mut state, &mut anchors, &rules)?;

    Ok(anchors)
}

fn parse_impl(state: &mut ParseState, anchors: &mut Vec<Anchor>, rules: &Rules) -> Result<()> {
    state.recursion_depth += 1;
    log::debug!("{state:?}");

    match state.state {
        // Contains the base-case
        ParseStateMachine::Base => {
            for i in state.cursor..state.tokens.len() {
                state.cursor = i;
                let Some(ch) = state.tokens.get(i).copied() else {
                    return Ok(());
                };
                if ch == ESCAPE {
                    state.state = ParseStateMachine::Escaping;
                    return parse_impl(state, anchors, rules);
                } else if ch == ANCHOR {
                    state.state = ParseStateMachine::AnchorBegin;
                    state.bound_anchor = Some(Anchor {
                        start: state.cursor,
                        ..Default::default()
                    });
                    return parse_impl(state, anchors, rules);
                }
            }
            Ok(())
        }

        ParseStateMachine::Escaping => {
            state.cursor += 1;
            if state.tokens.get(state.cursor).is_none() {
                return Err(ParseError::missing_escapee(state.cursor - 1, &state.tokens).into());
            }
            state.cursor += 1;
            state.state = ParseStateMachine::Base;
            parse_impl(state, anchors, rules)
        }

        ParseStateMachine::AnchorBegin => {
            state.cursor += 1;
            if state
                .tokens
                .get(state.cursor)
                .is_none_or(|ch| *ch != ANCHOR_OPEN)
            {
                return Err(
                    ParseError::invalid_anchor_start(state.cursor - 1, &state.tokens).into(),
                );
            }
            state.cursor += 1;

            for i in state.cursor..state.tokens.len() {
                if state
                    .tokens
                    .get(state.cursor)
                    .is_some_and(|ch| ch.is_ascii_whitespace())
                {
                    state.cursor = i;
                    break;
                }
                continue;
            }
            if state
                .tokens
                .get(state.cursor)
                .is_some_and(|ch| *ch == ATTRIBUTE_OPEN)
            {
                state.state = ParseStateMachine::AttributeParse;
            } else {
                state.state = ParseStateMachine::AnchorParseBase;
            }
            parse_impl(state, anchors, rules)
        }

        ParseStateMachine::AnchorParseBase => {
            let begin = state.cursor;
            let mut end = state.cursor;

            let Some(anchor) = state.bound_anchor.as_mut() else {
                log::error!("expected state.bound_anchor to be `Some` while in `AnchorParseBase`");
                return Err(format_err!(
                    "An unexpected error occurred while parsing template string."
                ));
            };

            for i in state.cursor..state.tokens.len() {
                state.cursor = i;
                let Some(ch) = state.tokens.get(state.cursor).copied() else {
                    return Err(ParseError::unclosed_anchor(state.cursor - 1, &state.tokens).into());
                };
                if ch == INDEX_OPEN {
                    for ch in &state.tokens[begin..end] {
                        if ch.is_ascii_whitespace() {
                            continue;
                        }
                        anchor.name.push(*ch)
                    }
                    let prev_token = state.tokens.get(state.cursor - 1).copied();

                    if anchor.name.is_empty()
                        || anchor
                            .name
                            .chars()
                            .last()
                            .is_some_and(|c| Some(c) != prev_token)
                    {
                        return Err(ParseError::invalid_indexing_operation(
                            state.cursor - 1,
                            &state.tokens,
                        )
                        .into());
                    }
                    state.state = ParseStateMachine::AnchorParseIndex;
                    break;
                } else if ch == ANCHOR_CLOSE {
                    log::debug!(
                        "FOOBAR: {}",
                        state.tokens[begin..end].iter().collect::<String>()
                    );
                    for ch in &state.tokens[begin..end] {
                        if ch.is_ascii_whitespace() {
                            continue;
                        }
                        anchor.name.push(*ch)
                    }
                    if anchor.name.is_empty() || !rules.valid_anchor_name.is_match(&anchor.name) {
                        return Err(ParseError::invalid_anchor_name(
                            state.cursor - 1,
                            &state.tokens,
                        )
                        .into());
                    }
                    state.state = ParseStateMachine::Base;

                    let Some(mut anchor) = state.bound_anchor.take() else {
                        log::error!("expected state.bound_anchor to be `Some` just before completing `AnchorParseBase`");
                        return Err(format_err!(
                            "An unexpected error occurred while parsing template string."
                        ));
                    };
                    anchor.end = state.cursor + 1;
                    anchors.push(anchor);
                    break;
                } else if ch == DEFAULT_PIPE {
                    for ch in &state.tokens[begin..end] {
                        if ch.is_ascii_whitespace() {
                            continue;
                        }
                        anchor.name.push(*ch)
                    }
                    state.cursor += 1;
                    let next_token_is_pipe = state
                        .tokens
                        .get(state.cursor)
                        .is_some_and(|ch| *ch == DEFAULT_PIPE);

                    if anchor.name.is_empty() || !next_token_is_pipe {
                        return Err(ParseError::invalid_default_value_operation(
                            state.cursor - 1,
                            &state.tokens,
                        )
                        .into());
                    }
                    state.state = ParseStateMachine::AnchorParseDefaultValue;
                    break;
                }
                end += 1;
            }
            parse_impl(state, anchors, rules)
        }

        ParseStateMachine::AnchorParseIndex => {
            state.cursor += 1;
            let begin = state.cursor;
            let mut end = begin;

            let Some(anchor) = state.bound_anchor.as_mut() else {
                log::error!("expected state.bound_anchor to be `Some` during `AnchorParseIndex`");
                return Err(format_err!(
                    "An unexpected error occurred while parsing template string."
                ));
            };
            for i in state.cursor..state.tokens.len() {
                state.cursor = i;
                if state.tokens.get(i).is_some_and(|ch| *ch == INDEX_CLOSE) {
                    state.cursor += 1;
                    break;
                }
                end += 1;
            }
            let index = state.tokens[begin..end]
                .iter()
                .collect::<String>()
                .parse::<usize>()
                .map_err(|_| ParseError::invalid_index(state.cursor - 1, &state.tokens))?;
            anchor.index = Some(index);

            state.state = ParseStateMachine::AnchorParseBase;
            parse_impl(state, anchors, rules)
        }

        ParseStateMachine::AnchorParseDefaultValue => {
            state.cursor += 1;
            for i in state.cursor..state.tokens.len() {
                state.cursor = i;

                let Some(ch) = state.tokens.get(state.cursor).copied() else {
                    break;
                };
                if ch.is_ascii_whitespace() {
                    continue;
                } else if ch == LITERAL_SINGLE_QUOTE || ch == LITERAL_DOUBLE_QUOTE {
                    state.cursor -= 1;
                    state.state = ParseStateMachine::AnchorParseDefaultLiteral;
                    return parse_impl(state, anchors, rules);
                } else if rules.valid_anchor_name.is_match(&ch.to_string()) {
                    state.cursor -= 1;
                    state.state = ParseStateMachine::AnchorParseDefaultAnchor;
                    return parse_impl(state, anchors, rules);
                } else {
                    return Err(ParseError::default_parsing_disallowed_char(
                        state.cursor - 1,
                        &state.tokens,
                    )
                    .into());
                }
            }
            Err(ParseError::default_parsing_eol(state.cursor - 1, &state.tokens).into())
        }

        ParseStateMachine::AnchorParseDefaultLiteral => {
            for i in state.cursor..state.tokens.len() {
                state.cursor = i;
                if state
                    .tokens
                    .get(i)
                    .copied()
                    .is_some_and(|ch| ch.is_ascii_whitespace())
                {
                    continue;
                }
                break;
            }
            let Some(opening_quote) = state.tokens.get(state.cursor).copied() else {
                log::error!("expected opening quote that starts the default literal in `AnchorParseDefaultLiteral`");
                return Err(format_err!(
                    "An unexpected error occurred while parsing template string."
                ));
            };
            state.cursor += 1;

            let begin = state.cursor;
            let mut end = state.cursor + 1;

            for _ in state.cursor..state.tokens.len() {
                state.cursor += 1;

                let Some(ch) = state.tokens.get(state.cursor).copied() else {
                    break;
                };
                if ch == ESCAPE {
                    state.cursor += 1;
                    continue;
                } else if ch == opening_quote {
                    let Some(bound_anchor) = state.bound_anchor.as_mut() else {
                        log::error!("expected state.bound_anchor to be `Some` during `AnchorParseDefaultLiteral`");
                        return Err(format_err!(
                            "An unexpected error occurred while parsing template string."
                        ));
                    };
                    let literal_value = state.tokens[begin..end].iter().collect::<String>();
                    bound_anchor
                        .defaults
                        .push(DefaultValue::Literal(literal_value));
                    state.state = ParseStateMachine::AnchorParseBase;
                    state.cursor += 1;
                    return parse_impl(state, anchors, rules);
                }
                end += 1;
            }
            Err(ParseError::default_str_literal_missing_closing_quote(
                state.cursor - 1,
                &state.tokens,
            )
            .into())
        }

        ParseStateMachine::AnchorParseDefaultAnchor => {
            state.cursor += 1;
            for i in state.cursor..state.tokens.len() {
                state.cursor = i;
                if state
                    .tokens
                    .get(i)
                    .copied()
                    .is_some_and(|ch| ch.is_ascii_whitespace())
                {
                    continue;
                }
                break;
            }
            let begin = state.cursor;
            let mut end = begin + 1;
            let mut index = None;

            while state.cursor < state.tokens.len() {
                state.cursor += 1;

                let Some(ch) = state.tokens.get(state.cursor).copied() else {
                    break;
                };
                if index.is_some()
                    || ch.is_ascii_whitespace()
                    || ch == DEFAULT_PIPE
                    || ch == ANCHOR_CLOSE
                {
                    let name: String = state.tokens[begin..end].iter().collect();
                    if !rules.valid_anchor_name.is_match(&name) {
                        return Err(ParseError::invalid_anchor_name(
                            state.cursor - 1,
                            &state.tokens,
                        )
                        .into());
                    }
                    let Some(anchor) = state.bound_anchor.as_mut() else {
                        log::error!("expected state.bound_anchor to be `Some` while in `AnchorParseDefaultAnchor`");
                        return Err(format_err!(
                            "An unexpected error occurred while parsing template string."
                        ));
                    };
                    anchor.defaults.push(DefaultValue::Anchor { name, index });
                    state.state = ParseStateMachine::AnchorParseBase;
                    return parse_impl(state, anchors, rules);
                } else if ch == INDEX_OPEN {
                    state.cursor += 1;
                    let index_begin = state.cursor;
                    let mut index_end = index_begin + 1;

                    for _ in state.cursor..state.tokens.len() {
                        state.cursor += 1;
                        let Some(ch) = state.tokens.get(state.cursor).copied() else {
                            return Err(ParseError::index_parsing_eol(
                                state.cursor - 1,
                                &state.tokens,
                            )
                            .into());
                        };
                        if ch == INDEX_CLOSE {
                            break;
                        }
                        index_end += 1;
                    }
                    let parsed_index = state.tokens[index_begin..index_end]
                        .iter()
                        .collect::<String>()
                        .parse::<usize>()
                        .map_err(|_| ParseError::invalid_index(state.cursor - 1, &state.tokens))?;
                    index = Some(parsed_index);
                    continue;
                }
                end += 1;
            }
            Err(ParseError::default_parsing_eol(state.cursor - 1, &state.tokens).into())
        }

        ParseStateMachine::AttributeParse => {
            todo!()
        }
    }
}

impl Debug for ParseState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ParseState {
            recursion_depth,
            cursor,
            state,
            bound_anchor,
            ..
        } = self;
        write!(f, "ParseState: depth={recursion_depth} cursor={cursor} state={state:?}, bound_anchor={bound_anchor:?}")
    }
}
