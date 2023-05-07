use std::{fmt::Display, fs::File};

use csv::{Reader, ReaderBuilder};

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
use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub entry_type: EntryType,
    pub timestamp: DateTime<Local>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EntryType {
    #[serde(rename = "in")]
    ClockIn,
    #[serde(rename = "out")]
    ClockOut,
}

impl EntryType {
    pub fn colored(&self) -> String {
        use owo_colors::OwoColorize;
        match self {
            EntryType::ClockIn => "in".green().to_string(),
            EntryType::ClockOut => "out".red().to_string(),
        }
    }
}

impl Display for EntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryType::ClockIn => write!(f, "in"),
            EntryType::ClockOut => write!(f, "out"),
        }
    }
}

pub fn build_reader() -> Result<Reader<File>> {
    check_data_file()?;
    build_reader_inner()
}

fn build_reader_inner() -> Result<Reader<File>> {
    let data_file = CONFIG.get_output_file();
    ReaderBuilder::new()
        .has_headers(true)
        .from_path(&data_file)
        .wrap_err(ERR_READ_CSV(&data_file))
        .suggestion(SUGG_REPORT_ISSUE)
}

fn check_data_file() -> Result<()> {
    let mut reader = build_reader_inner()?;

    let de = reader.deserialize::<Entry>();

    let errs = de.filter_map(Result::err).collect::<Vec<_>>();

    if !errs.is_empty() {
        error!("Malformed CSV entries:");
        for err in errs {
            error!("{err}");
        }
        return Err(eyre!(
            "There are malformed entries in the CSV file. Please fix them manually and try again."
        ));
    }

    Ok(())
}
