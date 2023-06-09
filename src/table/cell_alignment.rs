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
use comfy_table::CellAlignment as CTCellAlignment;

// reimplements the CellAlignment enum with ValueEnum so that it can be used in clap

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CellAlignment {
    Left,
    Right,
    Center,
}

impl CellAlignment {
    pub fn get(&self) -> CTCellAlignment {
        match self {
            CellAlignment::Left => CTCellAlignment::Left,
            CellAlignment::Right => CTCellAlignment::Right,
            CellAlignment::Center => CTCellAlignment::Center,
        }
    }
}
