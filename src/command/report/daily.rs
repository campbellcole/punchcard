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

use chrono::Datelike;
use polars::{
    lazy::dsl::GetOutput,
    prelude::{Duration, *},
    series::ops::NullBehavior,
};

use crate::prelude::*;

use super::{
    map_datetime_to_date_str, ReportSettings, COL_DURATION, COL_ENTRY_TYPE, COL_TIMESTAMP,
    TIME_UNIT,
};

const RES_TOTAL_HOURS: &str = "Total Hours";
const RES_DATE: &str = "Date";
const RES_AVERAGE_SHIFT_DURATION: &str = "Avg. Shift Duration";
const RES_SHIFTS: &str = "Number of Shifts";

#[instrument]
pub fn generate_daily_report(cli_args: &Cli, settings: &ReportSettings) -> Result<LazyFrame> {
    let map_fn = super::map_fn!(settings);

    let now = Local::now();
    let days_to_subtract = now.weekday().num_days_from_monday();
    let last_monday = now - chrono::Duration::days(days_to_subtract as i64);

    #[allow(deprecated)]
    let this_week_start = last_monday.date().and_hms_opt(0, 0, 0).unwrap();
    let this_week_end = this_week_start + chrono::Duration::days(7);

    let df = new_reader(cli_args)?
        .select([
            col(COL_ENTRY_TYPE),
            col(COL_TIMESTAMP)
                .str()
                .strptime(
                    DataType::Datetime(TIME_UNIT, None),
                    StrptimeOptions {
                        format: Some(CSV_DATETIME_FORMAT.into()),
                        exact: true,
                        utc: true,
                        tz_aware: true,
                        cache: false,
                        strict: true,
                    },
                )
                .cast(DataType::Datetime(
                    TIME_UNIT,
                    Some("America/Los_Angeles".into()),
                )),
        ])
        .sort(
            COL_TIMESTAMP,
            SortOptions {
                descending: false,
                nulls_last: false,
                multithreaded: true,
            },
        )
        .with_column(
            col(COL_TIMESTAMP)
                .diff(1, NullBehavior::Ignore)
                .alias(COL_DURATION),
        )
        .filter(
            col(COL_TIMESTAMP)
                .gt_eq(lit(this_week_start.timestamp_nanos()))
                .and(col(COL_TIMESTAMP).lt(lit(this_week_end.timestamp_nanos()))),
        )
        .filter(col(COL_ENTRY_TYPE).eq(lit("out")))
        .groupby_dynamic(
            col(COL_TIMESTAMP),
            [],
            DynamicGroupOptions {
                every: Duration::parse("1d"),
                period: Duration::parse("1d"),
                offset: Duration::parse("0d"),
                index_column: COL_TIMESTAMP.into(),
                start_by: StartBy::WindowBound,
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
            col(COL_TIMESTAMP).alias(RES_DATE).map(
                map_datetime_to_date_str,
                GetOutput::from_type(DataType::Utf8),
            ),
            col(RES_TOTAL_HOURS).map(map_fn, GetOutput::from_type(DataType::Utf8)),
            col(RES_SHIFTS),
            (col(RES_TOTAL_HOURS) / col(RES_SHIFTS))
                .alias(RES_AVERAGE_SHIFT_DURATION)
                .cast(DataType::Duration(TIME_UNIT))
                .map(map_fn, GetOutput::from_type(DataType::Utf8)),
        ]);

    Ok(df)
}
