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
use std::{fs, process::exit};

use clap::{CommandFactory, Parser, Subcommand};
#[cfg(feature = "generate_test_data")]
use data::generate::GenerateDataArgs;
use data::{
    clock::{ClockEntryArgs, EntryType},
    report::GenerateReportArgs,
};

use env::CONFIG;

#[macro_use]
extern crate serde;

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

// RFC3339 with nanoseconds, no space between ns and tz
const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S.%f%z";

pub mod biduration;
pub mod data;
pub mod env;
pub mod nlp;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub operation: Operation,
}

#[derive(Debug, Subcommand)]
pub enum Operation {
    /// Clock in
    #[command(name = "in")]
    ClockIn(ClockEntryArgs),
    /// Clock out
    #[command(name = "out")]
    ClockOut(ClockEntryArgs),
    /// Clock either in or out
    #[command(name = "toggle")]
    ClockToggle(ClockEntryArgs),
    /// Interpret the times and generate a report
    #[command(name = "report")]
    GenerateReport(GenerateReportArgs),
    /// Generate completions for the given shell
    #[command(name = "completions")]
    GenerateCompletions {
        #[clap(value_enum)]
        shell: clap_complete_command::Shell,
    },
    #[cfg(feature = "generate_test_data")]
    /// Generate test data
    GenerateData(GenerateDataArgs),
}

fn main() {
    dotenvy::dotenv().ok();

    let args = Cli::parse();

    if !CONFIG.data_folder().exists() {
        if let Err(err) = fs::create_dir_all(CONFIG.data_folder()) {
            eprintln!("Failed to create data folder: {}", err);
            exit(1);
        }
    }

    if let Err(err) = match args.operation {
        Operation::ClockIn(args) => data::clock::add_entry(EntryType::ClockIn, args),
        Operation::ClockOut(args) => data::clock::add_entry(EntryType::ClockOut, args),
        Operation::ClockToggle(args) => data::clock::toggle_clock(args),
        Operation::GenerateReport(args) => data::report::generate_report(args),
        Operation::GenerateCompletions { shell } => {
            shell.generate(&mut Cli::command(), &mut std::io::stdout());
            Ok(())
        }
        #[cfg(feature = "generate_test_data")]
        Operation::GenerateData(args) => data::generate::generate_test_entries(args),
    } {
        use owo_colors::{DynColors, OwoColorize};

        eprintln!(
            "{}{} {}",
            "Error".red().bold(),
            ":".color(DynColors::Rgb(128, 128, 128)),
            err
        );
        exit(1);
    }
}

// move this back up once the lint is fixed
#[cfg(test)]
mod tests;
