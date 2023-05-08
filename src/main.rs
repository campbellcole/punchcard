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
use std::{fs, path::PathBuf};

use crate::csv::EntryType;
use chrono_tz::Tz;
use clap::{CommandFactory, Parser, Subcommand};
use color_eyre::{eyre::Context, Help, Result};
#[cfg(feature = "generate_test_data")]
use command::generate::GenerateDataArgs;
use command::{clock::ClockEntryArgs, report::ReportSettings};
use prelude::SUGG_PROPER_PERMS;
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[macro_use]
extern crate serde;

#[macro_use]
extern crate tracing;

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

pub mod biduration;
pub mod command;
pub mod common;
pub mod csv;
mod prelude;
pub mod quantity;
pub mod table;

fn default_timezone() -> Tz {
    let tz = iana_time_zone::get_timezone()
        .expect("Could not determine local timezone. Please use the PUNCHCARD_TIMEZONE environment variable, or set the '--timezone' option.");
    tz.parse().expect("The timezone provided by your system could not be parsed into an IANA timezone. Please use the PUNCHCARD_TIMEZONE environment variable, or set the --timezone option.")
}

fn default_data_folder() -> PathBuf {
    dirs::data_dir().expect("Could not locate a suitable data directory. Please use the PUNCHCARD_DATA_FOLDER environment variable, or set the '--data-folder' option.").join("punchcard")
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(short, long, env = "PUNCHCARD_DATA_FOLDER", default_value_os_t = default_data_folder())]
    pub data_folder: PathBuf,
    #[clap(short, long, env = "PUNCHCARD_TIMEZONE", default_value_t = default_timezone())]
    pub timezone: Tz,
    #[clap(subcommand)]
    pub operation: Operation,
}

impl Cli {
    pub fn get_output_file(&self) -> PathBuf {
        self.data_folder.join("hours.csv")
    }
}

#[derive(Debug, Subcommand)]
pub enum Operation {
    /// Clock in
    ///
    /// Adds a clock-in entry to the data file with the current time,
    /// or the time given with the '-o' flag.
    #[command(name = "in")]
    ClockIn(ClockEntryArgs),
    /// Clock out
    ///
    /// Adds a clock-out entry to the data file with the current time,
    /// or the time given with the '-o' flag.
    #[command(name = "out")]
    ClockOut(ClockEntryArgs),
    /// Clock either in or out
    ///
    /// Clocks in or out depending on what was done last. Override
    /// the time used with the '-o' flag.
    #[command(name = "toggle")]
    ClockToggle(ClockEntryArgs),
    /// Check the current status
    ///
    /// Prints whether or not you are clocked in right now, and
    /// will also print when the next entry occurs, if applicable.
    /// You can also use the '-o' option to override
    /// the time checked, so you can check if you were/will be clocked
    /// in/out at a certain time.
    #[command(name = "status")]
    ClockStatus(ClockEntryArgs),
    /// Interpret the times and generate a report
    ///
    /// Processes the entries in the data file and generates a weekly report
    /// for the past 10 weeks. You can use the '-n' option to change the
    /// number of weeks to generate a report for. You can also use the '-o'
    /// option to save the report to a file alongside printing it to stdout.
    #[command(name = "report")]
    GenerateReport(ReportSettings),
    /// Generate completions for the given shell
    ///
    /// Prints completions to stdout. You will need to pipe these
    /// to a file, and where that file goes depends on your shell.
    #[command(name = "completions")]
    GenerateCompletions {
        #[clap(value_enum)]
        shell: clap_complete_command::Shell,
    },
    #[cfg(feature = "generate_test_data")]
    /// Generate test data
    GenerateData(GenerateDataArgs),
}

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true))
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("error")))
        .with(ErrorLayer::default())
        .init();
    color_eyre::install()?;

    let cli_args = Cli::parse();

    let data_folder = &cli_args.data_folder;
    if !data_folder.exists() {
        fs::create_dir_all(data_folder)
            .wrap_err("Failed to create data folder")
            .suggestion(SUGG_PROPER_PERMS(data_folder))?;
    }

    match &cli_args.operation {
        Operation::ClockIn(args) => command::clock::add_entry(&cli_args, EntryType::ClockIn, args)
            .wrap_err("Failed to clock in")?,
        Operation::ClockOut(args) => {
            command::clock::add_entry(&cli_args, EntryType::ClockOut, args)
                .wrap_err("Failed to clock out")?
        }
        Operation::ClockStatus(args) => command::status::get_clock_status(&cli_args, args)
            .wrap_err("Failed to check clock status")?,
        Operation::ClockToggle(args) => command::clock::toggle_clock(&cli_args, args)
            .wrap_err("Failed to toggle clock status")?,
        Operation::GenerateReport(args) => command::report::generate_report(&cli_args, args)
            .wrap_err("Failed to generate report")?,
        Operation::GenerateCompletions { shell } => {
            shell.generate(&mut Cli::command(), &mut std::io::stdout());
        }
        #[cfg(feature = "generate_test_data")]
        Operation::GenerateData(args) => command::generate::generate_test_entries(&cli_args, args)
            .wrap_err("Failed to generate test entries")?,
    }

    Ok(())
}

// move this back up once the lint is fixed
#[cfg(test)]
mod tests;
