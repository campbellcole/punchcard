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

use clap::{ArgAction, Args};

use crate::prelude::{NumCols, NumRows};

use super::{cell_alignment::CellAlignment, color::Color, style::TableStyle};

#[derive(Debug, Clone, Args)]
pub struct TableSettings {
    /// The maximum number of characters to display in a string column.
    #[clap(short = 't', long, default_value_t = 32)]
    pub string_truncate: usize,
    /// The maximum number of columns to display (or 'all').
    #[clap(short = 'c', long, default_value_t = NumCols::Some(10))]
    pub max_n_cols: NumCols,
    /// The maximum number of rows to display (or 'all').
    #[clap(short = 'r', long, default_value_t = NumRows::Some(10))]
    pub max_n_rows: NumRows,
    /// Hide the column names.
    #[clap(short = 'n', long, default_value_t = false)]
    pub hide_column_names: bool,
    /// Hide the data types.
    #[clap(short = 'd', long, default_value_t = true)]
    pub hide_data_types: bool,
    /// Show data types and column names inline.
    #[clap(short = 'i', long, default_value_t = false)]
    pub inline_data_types: bool,
    /// Hide the column separator.
    #[clap(short = 'e', long, default_value_t = false)]
    pub hide_column_separator: bool,
    /// The table style.
    #[clap(short = 's', long, value_enum, default_value_t = TableStyle::Utf8Full)]
    pub style: TableStyle,
    /// Use rounded corners.
    #[clap(short = 'f', long, default_value_t = true)]
    pub rounded_corners: bool,
    /// Use solid inner borders instead of dashed.
    #[clap(short = 'b', long, default_value_t = true)]
    pub solid_inner_borders: bool,
    /// Text alignment within cells.
    #[clap(short = 'a', long, value_enum, default_value_t = CellAlignment::Center)]
    pub cell_alignment: CellAlignment,
    /// The maximum width of the table (defaults to TTY width)
    #[clap(short = 'w', long, default_value = None)]
    pub width: Option<u16>,
    /// The color of the header cells on the table
    #[clap(long, default_value_t = Color::DarkMagenta)]
    pub header_color: Color,
    /// The color of each column in the table. Can be applied multiple times, only the first 5 will be used.
    #[clap(long, action = ArgAction::Append)]
    pub column_colors: Option<Vec<Color>>,
    /// Completely disable emitting ANSI escape codes. Useful for piping to other programs. Enabled automatically for copyable reports.
    #[clap(long, action = ArgAction::SetTrue)]
    pub no_color: bool,
}
