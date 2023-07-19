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

use polars::prelude::*;

// for some reason TimeZone needs to be explicitly imported
use crate::{
    prelude::{TimeZone, *},
    table::{settings::TableSettings, DataFrameDisplay},
};

mod copyable;
mod daily;
mod weekly;

const TIME_UNIT: TimeUnit = TimeUnit::Nanoseconds;

const COL_TIMESTAMP: &str = "timestamp";
const COL_ENTRY_TYPE: &str = "entry_type";
const COL_DURATION: &str = "duration";

#[derive(Debug, Args)]
pub struct ReportSettings {
    #[clap(subcommand)]
    pub report_type: Option<ReportType>,
    /// Save the report to a file, or '-' for stdout (ignores the '--num-rows' flag)
    #[clap(short = 'o', long, default_value = None)]
    pub output_file: Option<Destination>,
    /// Only print the table and nothing else
    #[clap(short = 'j', long, default_value_t = false)]
    pub just_table: bool,
    /// Print exact durations instead of rounded
    #[clap(long = "exact", default_value_t = false)]
    pub exact_durations: bool,
    /// Generate a page that copies the rich-text report to the clipboard
    #[clap(long = "copyable", default_value_t = false)]
    pub copyable: bool,
    #[clap(flatten)]
    pub table_settings: TableSettings,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ReportType {
    /// Generate a report by week for a given month
    Weekly(WeeklyReportArgs),
    /// Generate a report by day for the current week
    Daily,
}

impl Default for ReportType {
    fn default() -> Self {
        Self::Weekly(Default::default())
    }
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
                let duration_str = duration.to_friendly_absolute_string();
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

macro_rules! map_fn {
    ($settings:ident) => {
        if $settings.exact_durations {
            crate::command::report::map_duration_to_str_exact
        } else {
            crate::command::report::map_duration_to_str
        }
    };
}

pub(crate) use map_fn;

use self::weekly::WeeklyReportArgs;

fn map_datetime_to_date_str(s: Series) -> PolarsResult<Option<Series>> {
    Ok(Some(
        s.iter()
            .filter_map(|x| {
                let AnyValue::Datetime(epoch, time_unit, tz) = x else {
                    return None;
                };
                assert_eq!(time_unit, TIME_UNIT);
                assert!(tz.is_some());
                let naive = chrono::NaiveDateTime::from_timestamp_opt(
                    epoch / 1_000_000_000,
                    (epoch % 1_000_000_000) as u32,
                )
                .unwrap();
                Some(naive.format("%d %B %Y").to_string())
            })
            .collect(),
    ))
}

#[instrument]
pub fn generate_report(cli_args: &Cli, settings: &ReportSettings) -> Result<()> {
    let df = match &settings.report_type.as_ref().cloned().unwrap_or_default() {
        ReportType::Weekly(args) => weekly::generate_weekly_report(cli_args, settings, args)?,
        ReportType::Daily => daily::generate_daily_report(cli_args, settings)?,
    };

    if settings.copyable {
        return copyable::generate_copyable_report(df, settings);
    }

    let mut df = df.collect().wrap_err("Failed to process hours")?;

    let using_stdout = settings
        .output_file
        .as_ref()
        .map(|x| x.is_stdout())
        .unwrap_or(false);

    if !settings.just_table && !using_stdout {
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

    if !using_stdout {
        let display = DataFrameDisplay::new(&df, &settings.table_settings);
        println!("{display}");
    }

    if let Some(output_file) = &settings.output_file {
        let writer = output_file
            .to_writer()
            .wrap_err_with(|| ERR_OPEN_CSV(output_file.unwrap_path()))
            .with_suggestion(|| SUGG_PROPER_PERMS(output_file.unwrap_path()))?;
        CsvWriter::new(writer)
            .has_header(true)
            .finish(&mut df)
            .wrap_err_with(|| ERR_WRITE_CSV(output_file.unwrap_path()))?;
    }

    Ok(())
}
