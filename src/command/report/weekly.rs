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

use chrono::{Datelike, Timelike};
use polars::{
    prelude::{Duration, *},
    series::ops::NullBehavior,
};

use crate::prelude::*;

use super::{
    map_datetime_to_date_str, ReportSettings, COL_DURATION, COL_ENTRY_TYPE, COL_TIMESTAMP,
    TIME_UNIT,
};

const RES_TOTAL_HOURS: &str = "Total Hours";
const RES_WEEK_OF: &str = "Week Of";
const RES_WEEK_END: &str = "Week End";
const RES_AVERAGE_SHIFT_DURATION: &str = "Avg. Shift Duration";
const RES_SHIFTS: &str = "Number of Shifts";

#[derive(Debug, Clone, Args, Default)]
pub struct WeeklyReportArgs {
    #[clap(short, long, default_value_t = Default::default())]
    /// The month to generate the report for
    ///
    /// Accepts a month name (e.g. `January`) or a number (e.g. `1`)
    /// or `current`, `previous`, or `next`
    pub month: Month,
    #[clap(short, long, default_value_t = false)]
    /// Include shifts that occurred in a previous/upcoming month but
    /// spill in to or out of this month
    pub spill_over: bool,
}

#[instrument]
pub fn generate_weekly_report(
    cli_args: &Cli,
    settings: &ReportSettings,
    args: &WeeklyReportArgs,
) -> Result<LazyFrame> {
    let range = args.month.as_date().map(|month_start| {
        let month_end = {
            let mut date = month_start;
            date = date.with_month(month_start.month() + 1).unwrap();
            date -= chrono::Duration::days(1);
            date = date
                .with_hour(23)
                .unwrap()
                .with_minute(59)
                .unwrap()
                .with_second(59)
                .unwrap()
                .with_nanosecond(999_999_999)
                .unwrap();
            date
        };
        (month_start, month_end)
    });

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
                )
                // then we cast back to local time
                .cast(DataType::Datetime(
                    TIME_UNIT,
                    Some(cli_args.timezone.to_string()),
                )),
        ])
        .sort(
            COL_TIMESTAMP,
            SortOptions {
                descending: false,
                nulls_last: false,
                multithreaded: true,
                maintain_order: false,
            },
        )
        .with_column(
            col(COL_TIMESTAMP)
                .diff(1, NullBehavior::Ignore)
                .alias(COL_DURATION),
        )
        .filter(col(COL_ENTRY_TYPE).eq(lit("out")));

    if let Some((month_start, month_end)) = range {
        if !args.spill_over {
            df = df.filter(
                col(COL_TIMESTAMP)
                    .gt_eq(lit(month_start.timestamp_nanos()))
                    .and(col(COL_TIMESTAMP).lt(lit(month_end.timestamp_nanos()))),
            );
        }
    }

    df = df
        .groupby_dynamic(
            col(COL_TIMESTAMP),
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
                check_sorted: true,
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

    if let Some((month_start, month_end)) = range {
        if args.spill_over {
            // this will include any weeks which cross into or out of the month
            // the first condition checks if the week starts before the month starts
            // and ends after the month starts
            // the second condition checks if the week starts before the month ends
            // and ends after the month ends
            // the third condition checks if the week is fully contained within the month
            // which is the default behavior
            df = df.filter(
                col(RES_WEEK_OF)
                    .lt(lit(month_start.timestamp_nanos()))
                    .and(col(RES_WEEK_END).gt_eq(lit(month_start.timestamp_nanos())))
                    .or(col(RES_WEEK_OF)
                        .lt(lit(month_end.timestamp_nanos()))
                        .and(col(RES_WEEK_END).gt_eq(lit(month_end.timestamp_nanos()))))
                    .or(col(RES_WEEK_OF)
                        .gt_eq(lit(month_start.timestamp_nanos()))
                        .and(col(RES_WEEK_OF).lt(lit(month_end.timestamp_nanos())))),
            )
        }
    }

    if !settings.copyable {
        df = prepare_for_display(df, settings);
    }

    Ok(df)
}

pub fn prepare_for_display(df: LazyFrame, settings: &ReportSettings) -> LazyFrame {
    let map_fn = super::map_fn!(settings);

    df.select([
        col(RES_WEEK_OF).map(
            map_datetime_to_date_str,
            GetOutput::from_type(DataType::Utf8),
        ),
        col(RES_TOTAL_HOURS).map(map_fn, GetOutput::from_type(DataType::Utf8)),
        col(RES_WEEK_END).map(
            map_datetime_to_date_str,
            GetOutput::from_type(DataType::Utf8),
        ),
        col(RES_SHIFTS),
        col(RES_AVERAGE_SHIFT_DURATION).map(map_fn, GetOutput::from_type(DataType::Utf8)),
    ])
}
