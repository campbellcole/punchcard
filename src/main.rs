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

use std::error::Error;

use clap::{CommandFactory, Parser, Subcommand};
#[cfg(feature = "generate_test_data")]
use data::generate::GenerateDataArgs;
use data::{
    clock::{ClockEntryArgs, EntryType},
    report::GenerateReportArgs,
};

use env::CONFIG;
use tokio::fs;

// RFC3339 with nanoseconds, no space between ns and tz
const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S.%f%z";

#[macro_use]
extern crate serde;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    let args = Cli::parse();

    if !CONFIG.data_folder().exists() {
        fs::create_dir_all(&CONFIG.data_folder()).await?;
    }

    match args.operation {
        Operation::GenerateReport(args) => {
            data::report::generate_report(args).await?;
        }
        Operation::GenerateCompletions { shell } => {
            shell.generate(&mut Cli::command(), &mut std::io::stdout());
        }
        #[cfg(feature = "generate_test_data")]
        Operation::GenerateData(args) => {
            data::generate::generate_test_entries(args).await?;
        }
        Operation::ClockIn(args) => {
            data::clock::add_entry(EntryType::ClockIn, args).await?;
        }
        Operation::ClockOut(args) => {
            data::clock::add_entry(EntryType::ClockOut, args).await?;
        }
    }

    Ok(())
}

// move this back up once the lint is fixed
#[cfg(test)]
mod tests;
