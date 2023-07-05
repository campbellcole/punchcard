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

use std::str::FromStr;

use chrono::{Datelike, Local, Timelike};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Month {
    #[default]
    Current,
    Previous,
    Next,
    All,

    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl Month {
    pub fn as_date(&self) -> Option<chrono::DateTime<Local>> {
        use Month::*;
        let month_num = match self {
            All => return None,
            Current => Local::now().month(),
            Previous => {
                let mut date = Local::now();
                date = date.with_day(1).unwrap();
                date -= chrono::Duration::days(1);
                date.month()
            }
            Next => {
                let mut date = Local::now();
                date = date.with_day(1).unwrap();
                date += chrono::Duration::days(32);
                date.month()
            }
            January => 1,
            February => 2,
            March => 3,
            April => 4,
            May => 5,
            June => 6,
            July => 7,
            August => 8,
            September => 9,
            October => 10,
            November => 11,
            December => 12,
        };

        let mut date = Local::now();
        date = date.with_day(1).unwrap();
        date = date.with_month(month_num).unwrap();
        date = date
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();
        Some(date)
    }
}

#[derive(Debug, Error)]
pub enum ParseMonthError {
    #[error("Month {0} is not a valid month number")]
    InvalidMonthNumber(u8),
    #[error(
        "Unknown month {0}. Expected a month number, name, or 'current', 'previous', or 'next'"
    )]
    UnknownMonth(String),
}

impl FromStr for Month {
    type Err = ParseMonthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(num) = s.parse::<u8>() {
            match num {
                1 => Ok(Month::January),
                2 => Ok(Month::February),
                3 => Ok(Month::March),
                4 => Ok(Month::April),
                5 => Ok(Month::May),
                6 => Ok(Month::June),
                7 => Ok(Month::July),
                8 => Ok(Month::August),
                9 => Ok(Month::September),
                10 => Ok(Month::October),
                11 => Ok(Month::November),
                12 => Ok(Month::December),
                _ => Err(ParseMonthError::InvalidMonthNumber(num)),
            }
        } else {
            match s.to_lowercase().as_str() {
                "all" => Ok(Month::All),
                "current" => Ok(Month::Current),
                "previous" => Ok(Month::Previous),
                "next" => Ok(Month::Next),
                "january" => Ok(Month::January),
                "february" => Ok(Month::February),
                "march" => Ok(Month::March),
                "april" => Ok(Month::April),
                "may" => Ok(Month::May),
                "june" => Ok(Month::June),
                "july" => Ok(Month::July),
                "august" => Ok(Month::August),
                "september" => Ok(Month::September),
                "october" => Ok(Month::October),
                "november" => Ok(Month::November),
                "december" => Ok(Month::December),
                _ => Err(ParseMonthError::UnknownMonth(s.to_string())),
            }
        }
    }
}

impl ToString for Month {
    fn to_string(&self) -> String {
        use Month::*;
        match self {
            All => "all".into(),
            Current | Previous | Next => {
                // SAFETY: as_date() only returns None for All, so this is safe
                let date = self.as_date().unwrap();
                format!(
                    "{} ({})",
                    date.format("%B"),
                    match self {
                        Current => "current",
                        Previous => "previous",
                        Next => "next",
                        _ => unreachable!(),
                    }
                )
            }
            _ => {
                // SAFETY: as_date() only returns None for All, so this is safe
                let date = self.as_date().unwrap();
                date.format("%B").to_string()
            }
        }
    }
}
