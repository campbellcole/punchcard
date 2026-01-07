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

use chrono::{Datelike, Timelike};
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
const RES_WEEK_OF: &str = "Week Of";
const RES_WEEK_END: &str = "Week End";
const RES_AVERAGE_SHIFT_DURATION: &str = "Avg. Shift Duration";
const RES_SHIFTS: &str = "Number of Shifts";

#[derive(Debug, Clone, Args, Default)]
pub struct WeeklyReportArgs {
    /// The month to generate the report for
    ///
    /// Accepts a month name (e.g. `January`) or a number (e.g. `1`) or
    /// `current`, `previous`, or `next`
    #[clap(short, long, default_value_t = Default::default())]
    pub month: Month,
    /// The year to generate the report for
    ///
    /// Accepts an absolute year (e.g. `2026`) or a negative number offset (e.g.
    /// `-2`) which will be subtracted from the current year.
    #[clap(short, long, default_value = None)]
    pub year: Option<Year>,
    /// Include shifts that occurred in a previous/upcoming month but spill in
    /// to or out of this month
    #[clap(short, long, default_value_t = false)]
    pub spill_over: bool,
}

#[instrument]
pub fn generate_weekly_report(
    cli_args: &Cli,
    settings: &ReportSettings,
    args: &WeeklyReportArgs,
) -> Result<LazyFrame> {
    let range = args.month.as_date().map(|mut month_start| {
        let mut month_end = {
            let mut date = month_start;
            date = date.with_month((month_start.month() % 12) + 1).unwrap();

            // subtracting 1 day will get us to the last day of the previous
            // month however, in december this causes the year to roll back to
            // the previous year because `date`, before this line, is
            // <year>-01-01, so after this line it becomes <year-1>-12-31
            date -= chrono::Duration::days(1);

            // so we add the year back on if this happened
            if month_start.month() == 12 {
                date = date.with_year(date.year() + 1).unwrap();
            }

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

        if let Some(year) = args.year {
            let year = year.value();

            // SAFETY: `Year::value` is guaranteed to be representable by chrono
            month_start = month_start.with_year(year).unwrap();
            month_end = month_end.with_year(year).unwrap();
        }

        (month_start, month_end)
    });
    trace!(?range);

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
                // then we cast back to local time
                .cast(DataType::Datetime(
                    TIME_UNIT,
                    Some(TimeZone::from_chrono(&cli_args.timezone)),
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
        .filter(col(COL_ENTRY_TYPE).eq(lit("out")));

    if let Some((month_start, month_end)) = range
        && !args.spill_over
    {
        df = df.filter(
            col(COL_TIMESTAMP)
                .gt_eq(lit(month_start
                    .timestamp_nanos_opt()
                    .expect(NANOSECOND_OVERFLOW_MESSAGE)))
                .and(
                    col(COL_TIMESTAMP).lt(lit(month_end
                        .timestamp_nanos_opt()
                        .expect(NANOSECOND_OVERFLOW_MESSAGE))),
                ),
        );
    }

    df = df
        .group_by_dynamic(
            col(COL_TIMESTAMP),
            [],
            DynamicGroupOptions {
                every: Duration::parse("1w"),
                period: Duration::parse("1w"),
                offset: Duration::parse("0w"),
                index_column: COL_TIMESTAMP.into(),
                start_by: StartBy::Monday,
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
            col(COL_TIMESTAMP).alias(RES_WEEK_OF),
            col(RES_TOTAL_HOURS),
            (col(COL_TIMESTAMP) + lit(chrono::Duration::weeks(1))).alias(RES_WEEK_END),
            col(RES_SHIFTS),
            (col(RES_TOTAL_HOURS).cast(DataType::UInt64) / col(RES_SHIFTS))
                .alias(RES_AVERAGE_SHIFT_DURATION)
                .cast(DataType::Duration(TIME_UNIT)),
        ]);

    if let Some((month_start, month_end)) = range
        && args.spill_over
    {
        // this will include any weeks which cross into or out of the month the
        // first condition checks if the week starts before the month starts and
        // ends after the month starts the second condition checks if the week
        // starts before the month ends and ends after the month ends the third
        // condition checks if the week is fully contained within the month
        // which is the default behavior
        df = df.filter(
            col(RES_WEEK_OF)
                .lt(lit(month_start
                    .timestamp_nanos_opt()
                    .expect(NANOSECOND_OVERFLOW_MESSAGE)))
                .and(
                    col(RES_WEEK_END).gt_eq(lit(month_start
                        .timestamp_nanos_opt()
                        .expect(NANOSECOND_OVERFLOW_MESSAGE))),
                )
                .or(col(RES_WEEK_OF)
                    .lt(lit(month_end
                        .timestamp_nanos_opt()
                        .expect(NANOSECOND_OVERFLOW_MESSAGE)))
                    .and(
                        col(RES_WEEK_END).gt_eq(lit(month_end
                            .timestamp_nanos_opt()
                            .expect(NANOSECOND_OVERFLOW_MESSAGE))),
                    ))
                .or(col(RES_WEEK_OF)
                    .gt_eq(lit(month_start
                        .timestamp_nanos_opt()
                        .expect(NANOSECOND_OVERFLOW_MESSAGE)))
                    .and(
                        col(RES_WEEK_OF).lt(lit(month_end
                            .timestamp_nanos_opt()
                            .expect(NANOSECOND_OVERFLOW_MESSAGE))),
                    )),
        )
    }

    if !settings.copyable {
        df = prepare_for_display(df, settings);
    }

    Ok(df)
}

pub fn prepare_for_display(df: LazyFrame, settings: &ReportSettings) -> LazyFrame {
    let map_fn = super::map_fn!(settings);

    df.select([
        col(RES_WEEK_OF).map(map_datetime_to_date_str, super::coerce_output_type),
        col(RES_TOTAL_HOURS).map(map_fn, super::coerce_output_type),
        col(RES_WEEK_END).map(map_datetime_to_date_str, super::coerce_output_type),
        col(RES_SHIFTS),
        col(RES_AVERAGE_SHIFT_DURATION).map(map_fn, super::coerce_output_type),
    ])
}
