// Copyright (C) 2026 Campbell M. Cole
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
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::{num::ParseIntError, str::FromStr};

use chrono::{Datelike, Local, NaiveDate};
use thiserror::Error;

/// An `i32` with special parsing semantics. Positive numbers are parsed as is
/// and are provided directly in `self.0`, but negative numbers will be
/// subtracted from the current year.
///
/// For example, if it's 2026 and the string `"-2"` is parsed, this struct will
/// hold the value `2024`. If the string `"2020"` is parsed, regardless of year,
/// this struct will hold the value `2020`.
#[derive(Debug, Clone, Copy)]
pub struct Year(i32);

impl Year {
    /// The provided value is guaranteed to be a valid `chrono` year (i.e.
    /// `timelike.with_year(year.value())` will always return `Some`).
    pub fn value(&self) -> i32 {
        self.0
    }
}

#[derive(Debug, Error)]
pub enum ParseYearError {
    #[error("Invalid number '{0}': {1}")]
    InvalidNumber(String, ParseIntError),
    #[error("Impossible year: {0}")]
    ImpossibleYear(i64),
}

impl FromStr for Year {
    type Err = ParseYearError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn check_year_validity(year: i64) -> Result<i32, ParseYearError> {
            let year: i32 = year
                .try_into()
                .map_err(|_| ParseYearError::ImpossibleYear(year))?;

            NaiveDate::from_ymd_opt(year, 1, 1)
                .map(|_| year)
                .ok_or_else(|| ParseYearError::ImpossibleYear(year.into()))
        }

        let s = s.trim();

        if let Some(year) = s.strip_prefix('-') {
            // negative, current year minus what the user entered
            let year: u32 = year
                .parse()
                .map_err(|err| ParseYearError::InvalidNumber(year.to_string(), err))?;

            let year = check_year_validity(Local::now().year() as i64 - year as i64)?;

            Ok(Self(year))
        } else {
            let year: i32 = s
                .parse()
                .map_err(|err| ParseYearError::InvalidNumber(s.to_string(), err))?;

            let year = check_year_validity(year.into())?;

            Ok(Self(year))
        }
    }
}
