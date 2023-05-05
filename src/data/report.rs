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

use std::{fs::OpenOptions, path::PathBuf, str::FromStr};

use chrono::Local;
use clap::Args;
use polars::{
    lazy::dsl::{col, StrpTimeOptions},
    prelude::*,
    series::ops::NullBehavior,
};
use thiserror::Error;

use crate::env::CONFIG;

#[derive(Debug, Args)]
pub struct GenerateReportArgs {
    /// Save the report to a file (will save every row, ignoring the '--num-rows' flag)
    #[clap(short, long)]
    pub output_file: Option<PathBuf>,
    /// Print the last N rows of the report
    #[clap(short, long, default_value_t = NumRows::Some(10), value_parser = <NumRows as FromStr>::from_str)]
    pub num_rows: NumRows,
}

#[derive(Debug, Clone)]
pub enum NumRows {
    All,
    Some(usize),
}

impl ToString for NumRows {
    fn to_string(&self) -> String {
        match self {
            NumRows::All => "all".into(),
            NumRows::Some(num) => num.to_string(),
        }
    }
}

#[derive(Debug, Error)]
pub enum NumRowsError {
    #[error("Number of rows cannot be zero")]
    Zero,
    #[error("Unknown value. Must be a positive integer or \"all\"")]
    Unknown,
}

impl FromStr for NumRows {
    type Err = NumRowsError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<usize>() {
            Ok(num) if num == 0 => Err(NumRowsError::Zero),
            Ok(num) => Ok(NumRows::Some(num)),
            Err(_) if s.eq_ignore_ascii_case("all") => Ok(NumRows::All),
            Err(_) => Err(NumRowsError::Unknown),
        }
    }
}

const TIME_UNIT: TimeUnit = TimeUnit::Nanoseconds;

const COL_TIMESTAMP: &str = "timestamp";
const COL_ENTRY_TYPE: &str = "entry_type";
const COL_DURATION: &str = "duration";

const RES_TOTAL_HOURS: &str = "Total Hours";
const RES_WEEK_OF: &str = "Week Of";
const RES_WEEK_END: &str = "Week End";
const RES_AVERAGE_SHIFT_DURATION: &str = "Avg. Shift Duration";
const RES_SHIFTS: &str = "Number of Shifts";

pub fn generate_report(
    GenerateReportArgs {
        output_file,
        num_rows,
    }: GenerateReportArgs,
) -> PolarsResult<()> {
    let df = LazyCsvReader::new(CONFIG.get_output_file())
        .finish()?
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
                    Some(CONFIG.timezone().to_string()),
                )),
        ])
        .with_column(
            col(COL_TIMESTAMP)
                .diff(1, NullBehavior::Ignore)
                .alias(COL_DURATION),
        )
        .filter(col(COL_ENTRY_TYPE).eq(lit("out")))
        .sort(
            COL_TIMESTAMP,
            SortOptions {
                descending: false,
                nulls_last: false,
                multithreaded: true,
            },
        )
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
                .cast(DataType::Duration(TIME_UNIT)),
        ]);

    let mut df = df.collect()?;

    std::env::set_var("POLARS_FMT_TABLE_FORMATTING", "UTF8_FULL_CONDENSED");
    std::env::set_var("POLARS_FMT_TABLE_ROUNDED_CORNERS", "1");
    std::env::set_var("POLARS_FMT_TABLE_HIDE_COLUMN_DATA_TYPES", "1");
    std::env::set_var("POLARS_FMT_TABLE_CELL_ALIGNMENT", "center");
    std::env::set_var("POLARS_FMT_TABLE_HIDE_DATAFRAME_SHAPE_INFORMATION", "1");
    std::env::set_var("POLARS_FMT_MAX_ROWS", num_rows.to_string());

    println!("Report generated at {}:\n", Local::now());
    if let NumRows::Some(num) = num_rows {
        println!("{}", df.tail(Some(num)));
    } else {
        println!("{}", df);
    }

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
