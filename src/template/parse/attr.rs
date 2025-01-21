use super::super::token::{LITERAL_DOUBLE_QUOTE, LITERAL_SINGLE_QUOTE, PARAM_DELIMETER};
use anyhow::{format_err, Context, Result};
use crossterm::style::Stylize;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct Attribute {
    pub kind: AttributeKind,
    pub must_match: Option<Regex>,
}

/// Attributes that can be applied to an anchor.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AttributeKind {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BgBlack,
    BgRed,
    BgGreen,
    BgYellow,
    BgBlue,
    BgMagenta,
    BgCyan,
    BgWhite,
    Bold,
    Underlined,
    Reverse,
    CrossedOut,

    /// Comes in three flavors:
    /// - `lalign` (left align)
    /// - `ralign` (right align)
    /// - `calign` (center align)
    ///
    /// First and only argument is the width
    /// - `lalign(10)`
    Align {
        direction: Alignment,
        width: usize,
    },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Alignment {
    Left,
    Right,
    Center,
}

impl Attribute {
    /// TODO: Clean this up
    pub fn parse(val: String, raw_args: Option<String>) -> Result<Self> {
        let mut conditional = false;

        let attr_name = if let Some(stripped) = val.strip_prefix("?") {
            conditional = true;
            stripped.to_lowercase()
        } else {
            val.to_lowercase()
        };

        let args = raw_args.map_or_else(Vec::new, |rarg| {
            rarg.split(PARAM_DELIMETER)
                .map(|raws| {
                    raws.trim()
                        .trim_start_matches(LITERAL_SINGLE_QUOTE)
                        .trim_start_matches(LITERAL_DOUBLE_QUOTE)
                        .trim_end_matches(LITERAL_SINGLE_QUOTE)
                        .trim_end_matches(LITERAL_DOUBLE_QUOTE)
                        .to_string()
                })
                .collect::<Vec<String>>()
        });
        let mut args_iter = args.iter();
        let mut must_match = None;

        if conditional {
            if let Some(pattern) = args_iter.next() {
                let re = Regex::new(pattern).with_context(|| {
                    format!(
                        "expected first argument of conditional attribute to be a valid regular expression: {pattern}"
                    )
                })?;
                must_match = Some(re);
            } else {
                return Err(format_err!(
                    "Expected conditional attribute to have at least 1 argument."
                ));
            }
        }

        let kind = match attr_name.as_str() {
            "black" => AttributeKind::Black,
            "red" => AttributeKind::Red,
            "green" => AttributeKind::Green,
            "yellow" => AttributeKind::Yellow,
            "blue" => AttributeKind::Blue,
            "magenta" => AttributeKind::Magenta,
            "cyan" => AttributeKind::Cyan,
            "white" => AttributeKind::White,
            "bg_black" | "bg-black" => AttributeKind::BgBlack,
            "bg_red" | "bg-red" => AttributeKind::BgRed,
            "bg_green" | "bg-green" => AttributeKind::BgGreen,
            "bg_yellow" | "bg-yellow" => AttributeKind::BgYellow,
            "bg_blue" | "bg-blue" => AttributeKind::BgBlue,
            "bg_magenta" | "bg-magenta" => AttributeKind::BgMagenta,
            "bg_cyan" | "bg-cyan" => AttributeKind::BgCyan,
            "bg_white" | "bg-white" => AttributeKind::BgWhite,
            "bold" => AttributeKind::Bold,
            "underlined" => AttributeKind::Underlined,
            "reverse" => AttributeKind::Reverse,
            "crossedout" | "crossed_out" | "crossed-out" => AttributeKind::CrossedOut,
            "lalign" | "calign" | "ralign" => {
                let width = args_iter
                    .next()
                    .map(|w| {
                        w.parse::<usize>().map_err(|err| {
                            format_err!("expected first argument to '{attr_name}' to be a number: {err}")
                        })
                    })
                    .ok_or_else(|| format_err!("expected at least one argument for '{attr_name}'"))??;

                match attr_name.as_str() {
                    "lalign" => AttributeKind::Align {
                        direction: Alignment::Left,
                        width,
                    },
                    "calign" => AttributeKind::Align {
                        direction: Alignment::Center,
                        width,
                    },
                    "ralign" => AttributeKind::Align {
                        direction: Alignment::Right,
                        width,
                    },
                    _ => unreachable!(),
                }
            }
            _ => return Err(format_err!("unrecognized attribute '{val}'")),
        };

        Ok(Self { kind, must_match })
    }

    /// Applies select attributes to a given text data.
    pub fn apply(txt: &str, attributes: &[Self]) -> String {
        let mut val = txt.to_string().stylize();
        for attribute in attributes {
            if let Some(re) = attribute.must_match.as_ref() {
                if !re.is_match(txt) {
                    continue;
                }
            }
            val = match attribute.kind {
                AttributeKind::Black => val.black(),
                AttributeKind::Red => val.red(),
                AttributeKind::Green => val.green(),
                AttributeKind::Yellow => val.yellow(),
                AttributeKind::Blue => val.blue(),
                AttributeKind::Magenta => val.magenta(),
                AttributeKind::Cyan => val.cyan(),
                AttributeKind::White => val.white(),
                AttributeKind::Bold => val.bold(),
                AttributeKind::Underlined => val.underlined(),
                AttributeKind::Reverse => val.reverse(),
                AttributeKind::CrossedOut => val.crossed_out(),
                AttributeKind::BgBlack => val.on_black(),
                AttributeKind::BgRed => val.on_red(),
                AttributeKind::BgGreen => val.on_green(),
                AttributeKind::BgYellow => val.on_yellow(),
                AttributeKind::BgBlue => val.on_blue(),
                AttributeKind::BgMagenta => val.on_magenta(),
                AttributeKind::BgCyan => val.on_cyan(),
                AttributeKind::BgWhite => val.on_white(),
                AttributeKind::Align { direction, width } => match direction {
                    Alignment::Left => {
                        let val = val.to_string();
                        format!("{val:<width$}", width = width).stylize()
                    }
                    Alignment::Right => {
                        let val = val.to_string();
                        format!("{val:>width$}", width = width).stylize()
                    }
                    Alignment::Center => {
                        let val = val.to_string();
                        format!("{val:^width$}", width = width).stylize()
                    }
                },
            };
        }
        val.to_string()
    }
}
