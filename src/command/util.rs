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

use chrono::SecondsFormat;

use crate::prelude::*;

#[derive(Debug, Args)]
pub struct UtilityCommands {
    #[clap(subcommand)]
    subcommand: UtilityCommand,
}

#[derive(Debug, Subcommand)]
pub enum UtilityCommand {
    #[command(name = "now")]
    GetTimestamp {
        #[clap(long, short, default_value_t = false)]
        human_readable: bool,
    },
}

pub fn run_utility_command(args: &UtilityCommands) -> Result<()> {
    match args.subcommand {
        UtilityCommand::GetTimestamp { human_readable } => {
            let now = Local::now();

            if human_readable {
                println!("{}", now.format("%Y-%m-%d %H:%M:%S"));
            } else {
                println!("{}", now.to_rfc3339_opts(SecondsFormat::Nanos, false));
            }
        }
    }

    Ok(())
}
