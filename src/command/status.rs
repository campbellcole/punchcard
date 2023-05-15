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
use crate::{csv::build_reader, prelude::*};

use super::clock::ClockEntryArgs;

#[instrument]
pub fn get_clock_status(
    cli_args: &Cli,
    ClockEntryArgs {
        offset_from_now,
        category,
    }: &ClockEntryArgs,
) -> Result<()> {
    let is_now = offset_from_now.is_none();
    let current_time = {
        let now = Local::now();
        offset_from_now
            .as_ref()
            .map(|offset| now + **offset)
            .unwrap_or(now)
    };

    let status = get_clock_status_inner(cli_args, current_time)?;

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
                    "{}{}{}{}{}",
                    "You are clocked ".color(gray),
                    entry_type.colored().bold(),
                    if is_now {
                        String::new()
                    } else {
                        format!(
                            " {} {}",
                            "as of".color(gray),
                            status
                                .current_time
                                .format(SLIM_DATETIME)
                                .bold()
                                .yellow()
                                .to_string()
                        )
                    },
                    if let Some(until) = status.until {
                        let duration = until - status.current_time;
                        format!(
                            " {} {} {}{}{}",
                            "until".color(gray),
                            until.format(SLIM_DATETIME).bold().magenta(),
                            "(".color(gray),
                            // SAFETY: until is always after current_time
                            humantime::format_duration(duration.to_std().unwrap())
                                .bold()
                                .green(),
                            ")".color(gray)
                        )
                    } else {
                        String::new()
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
fn get_clock_status_inner(cli_args: &Cli, current_time: DateTime<Local>) -> Result<ClockStatus> {
    let output_file = cli_args.get_output_file();

    if !output_file.exists() {
        return Ok(ClockStatus {
            status_type: ClockStatusType::NoDataFile,
            current_time,
            until: None,
        });
    }

    let mut reader = build_reader(cli_args)?;
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
