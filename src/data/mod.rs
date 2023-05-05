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

use std::io;

use polars::prelude::PolarsError;
use thiserror::Error;

use self::clock::EntryError;

pub mod clock;
#[cfg(feature = "generate_test_data")]
pub mod generate;
pub mod report;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Adding entry: {0}")]
    EntryError(#[from] EntryError),
    #[error("Data processing: {0}")]
    PolarsError(#[from] PolarsError),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
}

pub type Result<T = ()> = std::result::Result<T, CommandError>;
