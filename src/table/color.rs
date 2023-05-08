// Copyright (C) 2023 Campbell M. Cole
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
use std::{fmt::Display, str::FromStr};

use owo_colors::DynColors;

use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Reset,
    Black,
    DarkGray,
    Red,
    DarkRed,
    Green,
    DarkGreen,
    Yellow,
    DarkYellow,
    Blue,
    DarkBlue,
    Magenta,
    DarkMagenta,
    Cyan,
    DarkCyan,
    White,
    Gray,
    Rgb { r: u8, g: u8, b: u8 },
    AnsiValue(u8),
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let as_str = match self {
            Self::AnsiValue(value) => format!("ansi({})", value),
            Self::Rgb { r, g, b } => format!("rgb({},{},{})", r, g, b),
            _ => format!("{:?}", self).to_ascii_lowercase(),
        };
        write!(f, "{}", as_str)
    }
}

impl From<Color> for comfy_table::Color {
    fn from(value: Color) -> Self {
        match value {
            Color::Reset => Self::Reset,
            Color::Black => Self::Black,
            Color::DarkGray => Self::DarkGrey,
            Color::Red => Self::Red,
            Color::DarkRed => Self::DarkRed,
            Color::Green => Self::Green,
            Color::DarkGreen => Self::DarkGreen,
            Color::Yellow => Self::Yellow,
            Color::DarkYellow => Self::DarkYellow,
            Color::Blue => Self::Blue,
            Color::DarkBlue => Self::DarkBlue,
            Color::Magenta => Self::Magenta,
            Color::DarkMagenta => Self::DarkMagenta,
            Color::Cyan => Self::Cyan,
            Color::DarkCyan => Self::DarkCyan,
            Color::White => Self::White,
            Color::Gray => Self::Grey,
            Color::Rgb { r, g, b } => Self::Rgb { r, g, b },
            Color::AnsiValue(num) => Self::AnsiValue(num),
        }
    }
}

impl From<Color> for DynColors {
    fn from(value: Color) -> Self {
        use owo_colors::AnsiColors;
        match value {
            Color::Reset => Self::Ansi(AnsiColors::Default),
            Color::Black => Self::Ansi(AnsiColors::Black),
            Color::DarkGray => Self::Ansi(AnsiColors::BrightBlack),
            Color::Red => Self::Ansi(AnsiColors::BrightRed),
            Color::DarkRed => Self::Ansi(AnsiColors::Red),
            Color::Green => Self::Ansi(AnsiColors::BrightGreen),
            Color::DarkGreen => Self::Ansi(AnsiColors::Green),
            Color::Yellow => Self::Ansi(AnsiColors::BrightYellow),
            Color::DarkYellow => Self::Ansi(AnsiColors::Yellow),
            Color::Blue => Self::Ansi(AnsiColors::BrightBlue),
            Color::DarkBlue => Self::Ansi(AnsiColors::Blue),
            Color::Magenta => Self::Ansi(AnsiColors::BrightMagenta),
            Color::DarkMagenta => Self::Ansi(AnsiColors::Magenta),
            Color::Cyan => Self::Ansi(AnsiColors::BrightCyan),
            Color::DarkCyan => Self::Ansi(AnsiColors::Cyan),
            Color::White => Self::Ansi(AnsiColors::BrightWhite),
            Color::Gray => Self::Ansi(AnsiColors::White),
            Color::Rgb { r, g, b } => Self::Rgb(r, g, b),
            // there is no DynColor implementation for raw ansi codes
            Color::AnsiValue(_) => Self::Ansi(AnsiColors::Default),
        }
    }
}

impl FromStr for Color {
    type Err = color_eyre::eyre::Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if s.starts_with('#') {
            let s = s.trim_start_matches('#');
            if s.len() == 6 {
                let r = u8::from_str_radix(&s[0..2], 16)?;
                let g = u8::from_str_radix(&s[2..4], 16)?;
                let b = u8::from_str_radix(&s[4..6], 16)?;
                Ok(Color::Rgb { r, g, b })
            } else if s.len() == 3 {
                let r = u8::from_str_radix(&s[0..1], 16)?;
                let g = u8::from_str_radix(&s[1..2], 16)?;
                let b = u8::from_str_radix(&s[2..3], 16)?;
                Ok(Color::Rgb { r, g, b })
            } else {
                Err(eyre!("Invalid hex code: {}", s))
            }
        } else if let Ok(num) = s.parse::<u8>() {
            Ok(Color::AnsiValue(num))
        } else {
            match s.to_ascii_lowercase().as_str() {
                "reset" => Ok(Color::Reset),
                "black" => Ok(Color::Black),
                "darkgray" | "darkgrey" => Ok(Color::DarkGray),
                "red" => Ok(Color::Red),
                "darkred" => Ok(Color::DarkRed),
                "green" => Ok(Color::Green),
                "darkgreen" => Ok(Color::DarkGreen),
                "yellow" => Ok(Color::Yellow),
                "darkyellow" => Ok(Color::DarkYellow),
                "blue" => Ok(Color::Blue),
                "darkblue" => Ok(Color::DarkBlue),
                "magenta" => Ok(Color::Magenta),
                "darkmagenta" => Ok(Color::DarkMagenta),
                "cyan" => Ok(Color::Cyan),
                "darkcyan" => Ok(Color::DarkCyan),
                "white" => Ok(Color::White),
                "gray" | "grey" => Ok(Color::Gray),
                _ => Err(eyre!("Invalid color: {}", s)),
            }
        }
    }
}
