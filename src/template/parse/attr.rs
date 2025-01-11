use anyhow::{format_err, Error};
use crossterm::style::{StyledContent, Stylize};
use std::str::FromStr;

/// Attributes that can be applied to an anchor.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Attribute {
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
}

impl Attribute {
    /// Applies select attributes to a given text data.
    pub fn apply<'a>(txt: &'a str, attributes: &[Self]) -> StyledContent<&'a str> {
        let mut val = txt.stylize();
        for attribute in attributes {
            val = match attribute {
                Self::Black => val.black(),
                Self::Red => val.red(),
                Self::Green => val.green(),
                Self::Yellow => val.yellow(),
                Self::Blue => val.blue(),
                Self::Magenta => val.magenta(),
                Self::Cyan => val.cyan(),
                Self::White => val.white(),
                Self::Bold => val.bold(),
                Self::Underlined => val.underlined(),
                Self::Reverse => val.reverse(),
                Self::CrossedOut => val.crossed_out(),
                Self::BgBlack => val.on_black(),
                Self::BgRed => val.on_red(),
                Self::BgGreen => val.on_green(),
                Self::BgYellow => val.on_yellow(),
                Self::BgBlue => val.on_blue(),
                Self::BgMagenta => val.on_magenta(),
                Self::BgCyan => val.on_cyan(),
                Self::BgWhite => val.on_white(),
            };
        }
        val
    }
}

impl FromStr for Attribute {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "black" => Ok(Attribute::Black),
            "red" => Ok(Attribute::Red),
            "green" => Ok(Attribute::Green),
            "yellow" => Ok(Attribute::Yellow),
            "blue" => Ok(Attribute::Blue),
            "magenta" => Ok(Attribute::Magenta),
            "cyan" => Ok(Attribute::Cyan),
            "white" => Ok(Attribute::White),
            "bg_black" | "bg-black" => Ok(Attribute::BgBlack),
            "bg_red" | "bg-red" => Ok(Attribute::BgRed),
            "bg_green" | "bg-green" => Ok(Attribute::BgGreen),
            "bg_yellow" | "bg-yellow" => Ok(Attribute::BgYellow),
            "bg_blue" | "bg-blue" => Ok(Attribute::BgBlue),
            "bg_magenta" | "bg-magenta" => Ok(Attribute::BgMagenta),
            "bg_cyan" | "bg-cyan" => Ok(Attribute::BgCyan),
            "bg_white" | "bg-white" => Ok(Attribute::BgWhite),
            "bold" => Ok(Attribute::Bold),
            "underlined" => Ok(Attribute::Underlined),
            "reverse" => Ok(Attribute::Reverse),
            "crossedout" | "crossed_out" | "crossed-out" => Ok(Attribute::CrossedOut),
            _ => Err(format_err!("unrecognized attribute '{s}'")),
        }
    }
}
