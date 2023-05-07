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
use std::fs;

use crate::csv::EntryType;
use clap::{CommandFactory, Parser, Subcommand};
use color_eyre::{eyre::Context, Help, Result};
#[cfg(feature = "generate_test_data")]
use command::generate::GenerateDataArgs;
use command::{clock::ClockEntryArgs, report::GenerateReportArgs};
use prelude::SUGG_PROPER_PERMS;
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use env::CONFIG;

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
pub mod env;
pub mod nlp;
mod prelude;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub operation: Operation,
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
    GenerateReport(GenerateReportArgs),
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

    let args = Cli::parse();

    let data_folder = CONFIG.data_folder();
    if !data_folder.exists() {
        fs::create_dir_all(data_folder)
            .wrap_err("Failed to create data folder")
            .suggestion(SUGG_PROPER_PERMS(data_folder))?;
    }

    match args.operation {
        Operation::ClockIn(args) => {
            command::clock::add_entry(EntryType::ClockIn, args).wrap_err("Failed to clock in")?
        }
        Operation::ClockOut(args) => {
            command::clock::add_entry(EntryType::ClockOut, args).wrap_err("Failed to clock out")?
        }
        Operation::ClockStatus(args) => {
            command::status::get_clock_status(args).wrap_err("Failed to check clock status")?
        }
        Operation::ClockToggle(args) => {
            command::clock::toggle_clock(args).wrap_err("Failed to toggle clock status")?
        }
        Operation::GenerateReport(args) => {
            command::report::generate_report(args).wrap_err("Failed to generate report")?
        }
        Operation::GenerateCompletions { shell } => {
            shell.generate(&mut Cli::command(), &mut std::io::stdout());
        }
        #[cfg(feature = "generate_test_data")]
        Operation::GenerateData(args) => command::generate::generate_test_entries(args)
            .wrap_err("Failed to generate test entries")?,
    }

    Ok(())
}

// move this back up once the lint is fixed
#[cfg(test)]
mod tests;
