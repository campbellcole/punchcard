// Copyright (C) 2023 Campbell M. Cole
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU Affero General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more
// details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use chrono::Datelike;
use polars::{
    prelude::{Duration, TimeZone, *},
    series::ops::NullBehavior,
};

use crate::prelude::*;

use super::{
    COL_DURATION, COL_ENTRY_TYPE, COL_TIMESTAMP, NANOSECOND_OVERFLOW_MESSAGE, ReportSettings,
    TIME_UNIT, map_datetime_to_date_str,
};

const RES_TOTAL_HOURS: &str = "Total Hours";
const RES_DATE: &str = "Date";
const RES_AVERAGE_SHIFT_DURATION: &str = "Avg. Shift Duration";
const RES_SHIFTS: &str = "Number of Shifts";

#[instrument]
pub fn generate_daily_report(cli_args: &Cli, settings: &ReportSettings) -> Result<LazyFrame> {
    let now = Local::now();
    let days_to_subtract = now.weekday().num_days_from_monday();
    let last_monday = now - chrono::Duration::days(days_to_subtract as i64);

    #[allow(deprecated)]
    let this_week_start = last_monday.date().and_hms_opt(0, 0, 0).unwrap();
    let this_week_end = this_week_start + chrono::Duration::days(7);

    let mut df = new_reader(cli_args)?
        .select([
            col(COL_ENTRY_TYPE),
            col(COL_TIMESTAMP)
                .str()
                .strptime(
                    DataType::Datetime(TIME_UNIT, None),
                    StrptimeOptions {
                        format: Some(CSV_DATETIME_FORMAT.into()),
                        exact: true,
                        cache: false,
                        strict: true,
                    },
                    lit("raise"),
                )
                .cast(DataType::Datetime(
                    TIME_UNIT,
                    Some(TimeZone::from_chrono(
                        &"America/Los_Angeles".parse().unwrap(),
                    )),
                )),
        ])
        .sort(
            [COL_TIMESTAMP],
            SortMultipleOptions {
                descending: vec![false],
                nulls_last: vec![false],
                multithreaded: true,
                maintain_order: false,
                limit: None,
            },
        )
        .with_column(
            col(COL_TIMESTAMP)
                .diff(lit(1), NullBehavior::Ignore)
                .alias(COL_DURATION),
        )
        .filter(
            col(COL_TIMESTAMP)
                .gt_eq(lit(this_week_start
                    .timestamp_nanos_opt()
                    .expect(NANOSECOND_OVERFLOW_MESSAGE)))
                .and(
                    col(COL_TIMESTAMP).lt(lit(this_week_end
                        .timestamp_nanos_opt()
                        .expect(NANOSECOND_OVERFLOW_MESSAGE))),
                ),
        )
        .filter(col(COL_ENTRY_TYPE).eq(lit("out")))
        .group_by_dynamic(
            col(COL_TIMESTAMP),
            [],
            DynamicGroupOptions {
                every: Duration::parse("1d"),
                period: Duration::parse("1d"),
                offset: Duration::parse("0d"),
                index_column: COL_TIMESTAMP.into(),
                start_by: StartBy::WindowBound,
                closed_window: ClosedWindow::Left,
                label: Label::Left,
                include_boundaries: false,
            },
        )
        .agg([
            col(COL_DURATION).sum().alias(RES_TOTAL_HOURS),
            col(COL_DURATION).count().alias(RES_SHIFTS),
        ])
        .select([
            col(COL_TIMESTAMP).alias(RES_DATE),
            col(RES_TOTAL_HOURS),
            col(RES_SHIFTS),
            (col(RES_TOTAL_HOURS).cast(DataType::UInt64) / col(RES_SHIFTS))
                .alias(RES_AVERAGE_SHIFT_DURATION)
                .cast(DataType::Duration(TIME_UNIT)),
        ]);

    if !settings.copyable {
        df = prepare_for_display(df, settings);
    }

    Ok(df)
}

pub fn prepare_for_display(df: LazyFrame, settings: &ReportSettings) -> LazyFrame {
    let map_fn = super::map_fn!(settings);

    df.select([
        col(RES_DATE).map(map_datetime_to_date_str, super::coerce_output_type),
        col(RES_TOTAL_HOURS).map(map_fn, super::coerce_output_type),
        col(RES_SHIFTS),
        col(RES_AVERAGE_SHIFT_DURATION).map(map_fn, super::coerce_output_type),
    ])
}
