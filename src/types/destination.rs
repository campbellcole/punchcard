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

use std::{
    convert::Infallible,
    fs::OpenOptions,
    io::{self, Write},
    path::PathBuf,
    str::FromStr,
};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub enum Destination {
    Stdout,
    File(PathBuf),
}

impl FromStr for Destination {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-" => Ok(Destination::Stdout),
            _ => Ok(Destination::File(PathBuf::from(s))),
        }
    }
}

impl Destination {
    pub fn to_writer(&self) -> Result<Box<dyn Write>, io::Error> {
        match self {
            Destination::Stdout => Ok(Box::new(io::stdout())),
            Destination::File(path) => Ok(Box::new(
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(path)?,
            )),
        }
    }

    pub fn is_stdout(&self) -> bool {
        matches!(self, Destination::Stdout)
    }

    pub fn unwrap_path(&self) -> &PathBuf {
        match self {
            Destination::Stdout => panic!("Cannot unwrap stdout"),
            Destination::File(path) => path,
        }
    }
}
