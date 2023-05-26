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

use clap::ValueEnum;

// store all comfy_table::presets::* in an enum so that it can be used in clap

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TableStyle {
    AsciiFull,
    AsciiFullCondensed,
    AsciiNoBorders,
    AsciiBordersOnly,
    AsciiBordersOnlyCondensed,
    AsciiHorizontalOnly,
    AsciiMarkdown,
    Utf8Full,
    Utf8FullCondensed,
    Utf8NoBorders,
    Utf8BordersOnly,
    Utf8HorizontalOnly,
    Nothing,
}

impl TableStyle {
    pub fn get_style(&self) -> &'static str {
        use comfy_table::presets::*;
        match self {
            TableStyle::AsciiFull => ASCII_FULL,
            TableStyle::AsciiFullCondensed => ASCII_FULL_CONDENSED,
            TableStyle::AsciiNoBorders => ASCII_NO_BORDERS,
            TableStyle::AsciiBordersOnly => ASCII_BORDERS_ONLY,
            TableStyle::AsciiBordersOnlyCondensed => ASCII_BORDERS_ONLY_CONDENSED,
            TableStyle::AsciiHorizontalOnly => ASCII_HORIZONTAL_ONLY,
            TableStyle::AsciiMarkdown => ASCII_MARKDOWN,
            TableStyle::Utf8Full => UTF8_FULL,
            TableStyle::Utf8FullCondensed => UTF8_FULL_CONDENSED,
            TableStyle::Utf8NoBorders => UTF8_NO_BORDERS,
            TableStyle::Utf8BordersOnly => UTF8_BORDERS_ONLY,
            TableStyle::Utf8HorizontalOnly => UTF8_HORIZONTAL_ONLY,
            TableStyle::Nothing => NOTHING,
        }
    }

    pub fn is_utf8(&self) -> bool {
        matches!(
            self,
            TableStyle::Utf8Full
                | TableStyle::Utf8FullCondensed
                | TableStyle::Utf8NoBorders
                | TableStyle::Utf8BordersOnly
                | TableStyle::Utf8HorizontalOnly
        )
    }
}
