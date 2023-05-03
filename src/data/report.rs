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

use std::{fs::OpenOptions, path::PathBuf};

use chrono::Local;
use clap::Args;
use polars::{
    lazy::dsl::{col, StrpTimeOptions},
    prelude::*,
    series::ops::NullBehavior,
};

use crate::env::CONFIG;

#[derive(Debug, Args)]
pub struct GenerateReportArgs {
    #[clap(short, long)]
    pub output_file: Option<PathBuf>,
    #[clap(short, long)]
    pub num_rows: Option<u32>,
}

pub async fn generate_report(args: GenerateReportArgs) -> PolarsResult<()> {
    tokio::task::spawn_blocking(|| generate_report_inner(args))
        .await
        .unwrap()
}

const TIME_UNIT: TimeUnit = TimeUnit::Nanoseconds;

const COL_TIMESTAMP: &str = "timestamp";
const COL_ENTRY_TYPE: &str = "entry_type";
const COL_DURATION: &str = "duration";

const RES_TOTAL_HOURS: &str = "total_hours";
const RES_WEEK_OF: &str = "week_of";
const RES_WEEK_END: &str = "week_end";
const RES_AVERAGE_SHIFT_DURATION: &str = "average_shift_duration";
const RES_SHIFTS: &str = "shifts";

fn generate_report_inner(
    GenerateReportArgs {
        output_file,
        num_rows,
    }: GenerateReportArgs,
) -> PolarsResult<()> {
    let num_rows = if output_file.is_some() {
        None // we want all rows
    } else {
        // we can only display 8 rows, so use that by default
        // when printing to stdout
        Some(num_rows.unwrap_or(8))
    };

    let mut df = LazyCsvReader::new(CONFIG.get_output_file())
        .finish()?
        .sort(
            COL_TIMESTAMP,
            SortOptions {
                descending: false,
                nulls_last: false,
                multithreaded: true,
            },
        )
        .select([
            col(COL_ENTRY_TYPE),
            col(COL_TIMESTAMP)
                .str()
                .strptime(StrpTimeOptions {
                    fmt: Some("%Y-%m-%dT%H:%M:%S.%f%z".into()),
                    exact: true,
                    // we have to use UTC because of PST/PDT
                    utc: true,
                    tz_aware: true,
                    date_dtype: DataType::Datetime(TIME_UNIT, None),
                    cache: false,
                    strict: true,
                })
                // then we cast back to local time
                .cast(DataType::Datetime(
                    TIME_UNIT,
                    Some("America/Los_Angeles".into()),
                )),
        ])
        .with_column(
            col(COL_TIMESTAMP)
                .diff(1, NullBehavior::Ignore)
                .alias(COL_DURATION),
        )
        .filter(col(COL_ENTRY_TYPE).eq(lit("out")))
        .groupby_dynamic(
            [],
            DynamicGroupOptions {
                every: Duration::parse("1w"),
                period: Duration::parse("1w"),
                offset: Duration::parse("0w"),
                index_column: COL_TIMESTAMP.into(),
                start_by: StartBy::Monday,
                closed_window: ClosedWindow::Left,
                truncate: true,
                include_boundaries: false,
            },
        )
        .agg([
            col(COL_DURATION).sum().alias(RES_TOTAL_HOURS),
            col(COL_DURATION).count().alias(RES_SHIFTS),
        ])
        .select([
            col(COL_TIMESTAMP).alias(RES_WEEK_OF),
            col(RES_TOTAL_HOURS),
            (col(COL_TIMESTAMP) + lit(chrono::Duration::weeks(1))).alias(RES_WEEK_END),
            col(RES_SHIFTS),
            (col(RES_TOTAL_HOURS) / col(RES_SHIFTS))
                .alias(RES_AVERAGE_SHIFT_DURATION)
                .cast(DataType::Duration(TIME_UNIT))
                .cast(DataType::Utf8),
        ]);

    if let Some(num_rows) = num_rows {
        df = df.tail(num_rows);
    }

    let mut df = df.collect()?;

    println!("Report as of {}:\n\n{:?}", Local::now(), df);

    if let Some(output_file) = output_file {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(output_file)?;
        CsvWriter::new(file).finish(&mut df)?;
    }

    Ok(())
}
