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

use clap::ArgAction;
use polars::{
    lazy::dsl::{col, GetOutput, StrptimeOptions},
    prelude::{Duration, *},
    series::ops::NullBehavior,
};

// for some reason TimeZone needs to be explicitly imported
use crate::{
    prelude::{TimeZone, *},
    table::{cell_alignment::CellAlignment, color::Color, style::TableStyle, DataFrameDisplay},
};

const TIME_UNIT: TimeUnit = TimeUnit::Nanoseconds;

const COL_CATEGORY: &str = "category";
const COL_TIMESTAMP: &str = "timestamp";
const COL_ENTRY_TYPE: &str = "entry_type";
const COL_DURATION: &str = "duration";

const RES_TOTAL_HOURS: &str = "Total Hours";
const RES_WEEK_OF: &str = "Week Of";
const RES_WEEK_END: &str = "Week End";
const RES_AVERAGE_SHIFT_DURATION: &str = "Avg. Shift Duration";
const RES_SHIFTS: &str = "Number of Shifts";

#[derive(Debug, Args)]
pub struct ReportSettings {
    /// Save the report to a file (will save every row, ignoring the '--num-rows' flag)
    #[clap(short = 'o', long, default_value = None)]
    pub output_file: Option<PathBuf>,
    /// Only print the table and nothing else
    #[clap(short = 'j', long, default_value_t = false)]
    pub just_table: bool,
    /// Print exact durations instead of rounded
    #[clap(long = "exact", default_value_t = false)]
    pub exact_durations: bool,
    /// The maximum number of characters to display in a string column.
    #[clap(short = 't', long, default_value_t = 32)]
    pub string_truncate: usize,
    /// The maximum number of columns to display (or 'all').
    #[clap(short = 'c', long, default_value_t = NumCols::Some(10), value_parser = <NumCols as FromStr>::from_str)]
    pub max_n_cols: NumCols,
    /// The maximum number of rows to display (or 'all').
    #[clap(short = 'r', long, default_value_t = NumRows::Some(10), value_parser = <NumRows as FromStr>::from_str)]
    pub max_n_rows: NumRows,
    /// Hide the column names.
    #[clap(short = 'n', long, default_value_t = false)]
    pub hide_column_names: bool,
    /// Hide the data types.
    #[clap(short = 'd', long, default_value_t = true)]
    pub hide_data_types: bool,
    /// Show data types and column names inline.
    #[clap(short = 'i', long, default_value_t = false)]
    pub inline_data_types: bool,
    /// Hide the column separator.
    #[clap(short = 'e', long, default_value_t = false)]
    pub hide_column_separator: bool,
    /// The table style.
    #[clap(short = 's', long, value_enum, default_value_t = TableStyle::Utf8Full)]
    pub style: TableStyle,
    /// Use rounded corners.
    #[clap(short = 'f', long, default_value_t = true)]
    pub rounded_corners: bool,
    /// Use solid inner borders instead of dashed.
    #[clap(short = 'b', long, default_value_t = true)]
    pub solid_inner_borders: bool,
    /// Text alignment within cells.
    #[clap(short = 'a', long, value_enum, default_value_t = CellAlignment::Center)]
    pub cell_alignment: CellAlignment,
    /// The maximum width of the table (defaults to TTY width)
    #[clap(short = 'w', long, default_value = None)]
    pub width: Option<u16>,
    /// The color of the header cells on the table
    #[clap(long, default_value_t = Color::DarkMagenta)]
    pub header_color: Color,
    /// The color of each column in the table. Can be applied multiple times, only the first 5 will be used.
    #[clap(long, action = ArgAction::Append)]
    pub column_colors: Option<Vec<Color>>,
}

fn map_duration_to_str(s: Series) -> PolarsResult<Option<Series>> {
    Ok(Some(
        s.iter()
            .filter_map(|x| {
                let AnyValue::Duration(duration, time_unit) = x else {
                    return None;
                };
                assert_eq!(time_unit, TIME_UNIT);
                let duration = chrono::Duration::nanoseconds(duration);
                let duration = BiDuration::new(duration);
                let duration_str = duration.to_friendly_hours_string();
                Some(duration_str)
            })
            .collect(),
    ))
}

fn map_duration_to_str_exact(s: Series) -> PolarsResult<Option<Series>> {
    Ok(Some(
        s.iter()
            .filter_map(|x| {
                let AnyValue::Duration(duration, time_unit) = x else {
                    return None;
                };
                assert_eq!(time_unit, TIME_UNIT);
                let duration = chrono::Duration::nanoseconds(duration);
                let duration = BiDuration::new(duration);
                let (duration, _) = duration.to_std_duration();
                let duration_str = humantime::format_duration(duration);
                Some(duration_str.to_string())
            })
            .collect(),
    ))
}

#[instrument]
pub fn generate_report(cli_args: &Cli, table_settings: &ReportSettings) -> Result<()> {
    let map_fn = if table_settings.exact_durations {
        map_duration_to_str_exact
    } else {
        map_duration_to_str
    };

    let df = LazyCsvReader::new(cli_args.get_output_file())
        .finish()
        .wrap_err("Failed to create lazy CSV reader")?
        .select([
            col(COL_ENTRY_TYPE),
            col(COL_TIMESTAMP)
                .str()
                .strptime(
                    DataType::Datetime(TIME_UNIT, None),
                    StrptimeOptions {
                        format: Some(CSV_DATETIME_FORMAT.into()),
                        exact: true,
                        // we have to use UTC because of PST/PDT, etc.
                        utc: true,
                        tz_aware: true,
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
            },
        )
        .agg([
            col(COL_DURATION).sum().alias(RES_TOTAL_HOURS),
            col(COL_DURATION).count().alias(RES_SHIFTS),
        ])
        .select([
            col(COL_TIMESTAMP).alias(RES_WEEK_OF),
            col(RES_TOTAL_HOURS).map(map_fn, GetOutput::from_type(DataType::Utf8)),
            (col(COL_TIMESTAMP) + lit(chrono::Duration::weeks(1))).alias(RES_WEEK_END),
            col(RES_SHIFTS),
            (col(RES_TOTAL_HOURS) / col(RES_SHIFTS))
                .alias(RES_AVERAGE_SHIFT_DURATION)
                .cast(DataType::Duration(TIME_UNIT))
                .map(map_fn, GetOutput::from_type(DataType::Utf8)),
        ]);

    let mut df = df.collect().wrap_err("Failed to process hours")?;

    if !table_settings.just_table {
        use owo_colors::{DynColors, OwoColorize};
        let dark_gray = DynColors::Rgb(128, 128, 128);
        println!(
            "{} {}{}",
            "Report generated at".color(dark_gray),
            Local::now().format(&format!(
                "{} {}{}{} {} {}",
                PRETTY_TIME.magenta().bold(),
                "(".color(dark_gray),
                format!(
                    "{}",
                    cli_args
                        .timezone
                        .offset_from_utc_date(&Utc::now().date_naive())
                        .abbreviation()
                )
                .blue(),
                ")".color(dark_gray),
                "on".color(dark_gray),
                PRETTY_DATE.cyan().bold(),
            )),
            ":".color(dark_gray)
        );
    }

    let display = DataFrameDisplay::new(&df, table_settings);
    println!("{display}");

    if let Some(output_file) = &table_settings.output_file {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(output_file)
            .wrap_err(ERR_OPEN_CSV(output_file))
            .suggestion(SUGG_PROPER_PERMS(output_file))?;
        CsvWriter::new(file)
            .finish(&mut df)
            .wrap_err(ERR_WRITE_CSV(output_file))?;
    }

    Ok(())
}
