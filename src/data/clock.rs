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

use std::{fmt::Display, fs::File, str::FromStr};

use chrono_tz::OffsetName;
use csv::{Reader, ReaderBuilder};
use itertools::Itertools;

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

#[derive(Debug, Args)]
pub struct ClockEntryArgs {
    /// The offset from the current time to use as the clock in/out time
    #[clap(short, long, value_parser = <BiDuration as FromStr>::from_str)]
    pub offset_from_now: Option<BiDuration>,
    // /// Natural language to parse using ChatGPT. Reads key from "OPENAI_API_KEY" environment variable.
    // #[clap(short, long)]
    // pub nlp: Option<String>,
}

#[instrument]
pub fn add_entry(
    entry_type: EntryType,
    ClockEntryArgs {
        offset_from_now,
        // nlp,
    }: ClockEntryArgs,
) -> Result<()> {
    let timestamp = {
        let now = Local::now();
        offset_from_now
            .as_ref()
            .map(|offset| now + **offset)
            .unwrap_or(now)
    };

    let data_file = CONFIG.get_output_file();

    let last_entry = get_last_entry().wrap_err(ERR_LATEST_ENTRY)?;

    // currently cannot allow entries before the latest entry
    // because that would add a lot of complexity to the code.
    // basically trying to avoid interpreting the entire file
    // to make sure that every in has a matching out. this
    // logic provides the same guarantee but is much simpler.
    match last_entry.as_ref().map(|e| e.timestamp) {
        Some(time) if time > timestamp => {
            return Err(eyre!(
                "Cannot add an entry before the latest entry at {}",
                time.format("%r on %A, %d %B %Y")
            ));
        }
        _ => {}
    }

    let last_op = last_entry.map(|e| e.entry_type);

    if matches!(last_op, Some(op) if op == entry_type) {
        return Err(eyre!("Already clocked {entry_type}"));
    }

    let entry = Entry {
        entry_type,
        timestamp,
    };

    {
        // this is in a block because owo_colors adds functions to almost every type
        // and it's super annoying to have it in scope all the time
        use owo_colors::{DynColors, OwoColorize};
        // print this before saving because we have to move it
        // and I'm trying to avoid unnecessary cloning
        let gray = DynColors::Rgb(128, 128, 128);
        let oparen = "(".color(gray);
        let cparen = ")".color(gray);

        println!(
            "{} {} {} {}{}",
            "Clocked".color(gray),
            entry.entry_type.colored().bold(),
            "@".color(gray),
            entry.timestamp.format(&format!(
                "{} {}{}{} {} {}",
                "%r".magenta().bold(),
                oparen,
                format!(
                    "{}",
                    CONFIG
                        .timezone()
                        .offset_from_utc_date(&Utc::now().date_naive())
                        .abbreviation()
                )
                .blue(),
                cparen,
                "on".color(gray),
                "%A, %d %B %Y".cyan().bold(),
            )),
            if let Some(offset) = offset_from_now {
                format!(
                    " {}{}{}",
                    oparen,
                    offset.to_friendly_string().yellow().bold(),
                    cparen
                )
                .yellow()
                .to_string()
            } else {
                String::new()
            },
        );
    }

    let has_headers = !data_file.exists();

    let file = File::options()
        .create(true)
        .append(true)
        .open(&data_file)
        .wrap_err(ERR_OPEN_CSV(&data_file))
        .suggestion(SUGG_PROPER_PERMS(&data_file))?;

    let mut writer = csv::WriterBuilder::default()
        .has_headers(has_headers)
        .from_writer(file);

    writer
        .serialize(entry)
        .wrap_err(ERR_WRITE_CSV(&data_file))
        .suggestion(SUGG_PROPER_PERMS(&data_file))?;

    Ok(())
}

#[instrument]
pub fn toggle_clock(args: ClockEntryArgs) -> Result<()> {
    let last_op = get_last_entry()
        .wrap_err(ERR_LATEST_ENTRY)?
        .map(|e| e.entry_type);

    match last_op {
        Some(EntryType::ClockIn) => add_entry(EntryType::ClockOut, args),
        _ => add_entry(EntryType::ClockIn, args),
    }
}

#[instrument]
fn get_last_entry() -> Result<Option<Entry>> {
    let data_file = CONFIG.get_output_file();

    if data_file.exists() {
        check_data_file()?;
        let mut reader = build_reader()?;
        let de = reader.deserialize::<Entry>();

        if let Some(last_entry) = de
            .filter_map(Result::ok)
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

#[derive(Debug, Args)]
pub struct ClockStatusArgs {
    #[clap(short = 't', long)]
    pub at_time: Option<DateTime<Local>>,
}

#[instrument]
pub fn get_clock_status(ClockStatusArgs { at_time }: ClockStatusArgs) -> Result<()> {
    let is_now = at_time.is_none();
    let current_time = at_time.unwrap_or_else(Local::now);

    let status = get_clock_status_inner(current_time)?;

    {
        use owo_colors::{DynColors, OwoColorize};
        let gray = DynColors::Rgb(128, 128, 128);
        match status.status_type {
            ClockStatusType::NoDataFile => {
                println!(
                    "{}",
                    "The data file does not exist! Start using punchcard to generate it.".red()
                );
            }
            ClockStatusType::Entry(entry_type) => {
                println!(
                    "{} {} {} {}{}",
                    "You are clocked".color(gray),
                    entry_type.colored().bold(),
                    "as of".color(gray),
                    if is_now {
                        "now".bold().yellow().to_string()
                    } else {
                        status
                            .current_time
                            .format("%r on %A, %d %B %Y")
                            .bold()
                            .yellow()
                            .to_string()
                    },
                    ".".color(gray)
                )
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum ClockStatusType {
    NoDataFile,
    Entry(EntryType),
}

impl ClockStatusType {
    pub fn as_string(&self) -> String {
        match self {
            ClockStatusType::NoDataFile => String::new(),
            ClockStatusType::Entry(e) => e.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClockStatus {
    status_type: ClockStatusType,
    current_time: DateTime<Local>,
    until: Option<DateTime<Local>>,
}

#[instrument]
fn get_clock_status_inner(current_time: DateTime<Local>) -> Result<ClockStatus> {
    let output_file = CONFIG.get_output_file();

    if !output_file.exists() {
        return Ok(ClockStatus {
            status_type: ClockStatusType::NoDataFile,
            current_time,
            until: None,
        });
    }

    check_data_file()?;
    let mut reader = build_reader()?;
    let mut de = reader.deserialize::<Entry>();

    let mut this_entry = None;
    let mut next_entry = None;

    // all entries will be Ok because the build_reader method throws
    // an error if there are any malformed entries
    while let Some(Ok(entry)) = de.next() {
        if entry.timestamp > current_time {
            next_entry = Some(entry);
            break;
        } else {
            this_entry = Some(entry);
        }
    }

    let status_type = ClockStatusType::Entry(
        this_entry
            .map(|e| e.entry_type)
            .unwrap_or(EntryType::ClockOut),
    );

    let until = next_entry.map(|e| e.timestamp);

    Ok(ClockStatus {
        status_type,
        current_time,
        until,
    })
}

fn build_reader() -> Result<Reader<File>> {
    let data_file = CONFIG.get_output_file();
    ReaderBuilder::new()
        .has_headers(true)
        .from_path(&data_file)
        .wrap_err(ERR_READ_CSV(&data_file))
        .suggestion(SUGG_REPORT_ISSUE)
}

fn check_data_file() -> Result<()> {
    let mut reader = build_reader()?;

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
