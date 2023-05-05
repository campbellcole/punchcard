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

use std::{fmt::Display, fs, fs::File, io, str::FromStr};

use chrono::{DateTime, Local};
use clap::Args;
use itertools::Itertools;
use thiserror::Error;

use crate::{biduration::BiDuration, env::CONFIG};

#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub entry_type: EntryType,
    pub timestamp: DateTime<Local>,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum EntryType {
    #[serde(rename = "in")]
    ClockIn,
    #[serde(rename = "out")]
    ClockOut,
}

impl Display for EntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryType::ClockIn => write!(f, "in"),
            EntryType::ClockOut => write!(f, "out"),
        }
    }
}

#[derive(Debug, Error)]
pub enum EntryError {
    #[error("Already clocked {0}")]
    AlreadyClocked(String),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    // #[error("NLP error: {0}")]
    // Nlp(#[from] crate::nlp::NlpError),
}

#[derive(Debug, Args)]
pub struct ClockEntryArgs {
    /// The offset from the current time to use as the clock in/out time
    #[clap(short, long, value_parser = <BiDuration as FromStr>::from_str)]
    pub offset_from_now: Option<BiDuration>,
    // /// Natural language to parse using ChatGPT. Reads key from "OPENAI_API_KEY" environment variable.
    // #[clap(short, long)]
    // pub nlp: Option<String>,
}

pub fn add_entry(
    entry_type: EntryType,
    ClockEntryArgs {
        offset_from_now,
        // nlp,
    }: ClockEntryArgs,
) -> Result<(), EntryError> {
    let timestamp = {
        let now = Local::now();
        offset_from_now.map(|offset| now + *offset).unwrap_or(now)
    };

    let data_file = CONFIG.get_output_file();

    let last_op = get_last_entry()?.map(|e| e.entry_type);

    if matches!(last_op, Some(op) if op == entry_type) {
        return Err(EntryError::AlreadyClocked(entry_type.to_string()));
    }

    let entry = Entry {
        entry_type,
        timestamp,
    };

    let parent_dir = data_file.parent().unwrap();

    if !parent_dir.exists() {
        fs::create_dir_all(parent_dir)?;
    }

    // print this before saving because we have to move it
    // and I'm trying to avoid unnecessary cloning
    println!("Clocked {} at {}", entry.entry_type, entry.timestamp);

    let has_headers = !data_file.exists();

    let file = File::options().create(true).append(true).open(&data_file)?;

    let mut writer = csv::WriterBuilder::default()
        .has_headers(has_headers)
        .from_writer(file);

    writer.serialize(entry)?;

    Ok(())
}

pub fn toggle_clock(args: ClockEntryArgs) -> Result<(), EntryError> {
    let last_op = get_last_entry()?.map(|e| e.entry_type);

    match last_op {
        Some(EntryType::ClockIn) => add_entry(EntryType::ClockOut, args),
        _ => add_entry(EntryType::ClockIn, args),
    }
}

fn get_last_entry() -> Result<Option<Entry>, EntryError> {
    let data_file = CONFIG.get_output_file();

    if data_file.exists() {
        let mut reader = csv::ReaderBuilder::default()
            .has_headers(true)
            .from_path(&data_file)?;

        if let Some(last_entry) = reader
            .deserialize::<Entry>()
            .filter_map(|e| e.ok())
            .sorted_by(|e1, e2| {
                e1.timestamp
                    .partial_cmp(&e2.timestamp)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .last()
        {
            Ok(Some(last_entry))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}
