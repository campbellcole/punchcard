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

use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::PathBuf,
};

use chrono::{prelude::*, Duration};
use clap::Args;
use rand::prelude::*;

use crate::{env::CONFIG, DATETIME_FORMAT};

#[derive(Debug, Args)]
pub struct GenerateDataArgs {
    /// The number of entries to generate
    #[clap(short, long)]
    pub count: Option<usize>,
    /// The path to output the CSV file
    #[clap(short, long)]
    pub output_file: Option<PathBuf>,
}

pub fn generate_test_entries(
    GenerateDataArgs { count, output_file }: GenerateDataArgs,
) -> super::Result {
    let mut prev_time = Local::now();
    // three and a half hours
    let base_offset = Duration::seconds(60 * 30 * 7);
    let mut rng = rand::thread_rng();

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_file.unwrap_or_else(|| CONFIG.get_output_file()))?;

    let mut writer = BufWriter::new(file);

    writer.write_all(b"entry_type,timestamp\n")?;

    for x in 0..count.unwrap_or(10_000) {
        let entry_type = if x % 2 == 0 { "in" } else { "out" };

        let timestamp = if x == 0 {
            prev_time
        } else {
            prev_time
                + Duration::seconds(
                    (base_offset.num_seconds() as f64 * rng.gen_range(0.0..2.0)) as i64,
                )
        };

        writer.write_all(
            format!("{},{}\n", entry_type, timestamp.format(DATETIME_FORMAT)).as_bytes(),
        )?;

        prev_time = timestamp;
    }

    writer.flush()?;

    Ok(())
}
