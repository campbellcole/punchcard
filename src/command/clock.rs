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

use std::fs::File;

use chrono_tz::OffsetName;
use itertools::Itertools;

use crate::prelude::*;

#[derive(Debug, Args)]
pub struct ClockEntryArgs {
    /// The offset from the current time to use as the clock in/out time
    #[clap(short, long)]
    pub offset_from_now: Option<BiDuration>,
}

#[instrument]
pub fn add_entry(
    cli_args: &Cli,
    entry_type: EntryType,
    ClockEntryArgs { offset_from_now }: &ClockEntryArgs,
) -> Result<()> {
    let timestamp = {
        let now = Local::now();
        offset_from_now
            .as_ref()
            .map(|offset| now + **offset)
            .unwrap_or(now)
    };

    let data_file = cli_args.get_output_file();

    let last_entry = get_last_entry(cli_args).wrap_err(ERR_LATEST_ENTRY)?;

    // currently cannot allow entries before the latest entry
    // because that would add a lot of complexity to the code.
    // basically trying to avoid interpreting the entire file
    // to make sure that every in has a matching out. this
    // logic provides the same guarantee but is much simpler.
    match last_entry.as_ref().map(|e| e.timestamp) {
        Some(time) if time > timestamp => {
            return Err(eyre!(
                "Cannot add an entry before the latest entry at {}",
                time.format(PRETTY_DATETIME)
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
                PRETTY_TIME.magenta().bold(),
                oparen,
                format!(
                    "{}",
                    cli_args
                        .timezone
                        .offset_from_utc_date(&Utc::now().date_naive())
                        .abbreviation()
                )
                .blue(),
                cparen,
                "on".color(gray),
                PRETTY_DATE.cyan().bold(),
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
pub fn toggle_clock(cli_args: &Cli, args: &ClockEntryArgs) -> Result<()> {
    let last_op = get_last_entry(cli_args)
        .wrap_err(ERR_LATEST_ENTRY)?
        .map(|e| e.entry_type);

    match last_op {
        Some(EntryType::ClockIn) => add_entry(cli_args, EntryType::ClockOut, args),
        _ => add_entry(cli_args, EntryType::ClockIn, args),
    }
}

#[instrument]
fn get_last_entry(cli_args: &Cli) -> Result<Option<Entry>> {
    let data_file = cli_args.get_output_file();

    if data_file.exists() {
        let mut reader = build_reader(cli_args)?;
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
