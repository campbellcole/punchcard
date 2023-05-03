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
    ops::{Add, Deref},
    str::FromStr,
};

use chrono::{DateTime, Duration, TimeZone};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BiDuration(pub Duration);

impl Deref for BiDuration {
    type Target = Duration;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: TimeZone> Add<DateTime<T>> for BiDuration {
    type Output = DateTime<T>;
    fn add(self, rhs: DateTime<T>) -> Self::Output {
        rhs + *self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Forward,
    Backward,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Error)]
pub enum BiDurationParseError {
    #[error("invalid direction: {0}")]
    InvalidDirection(String),
    #[error("both forward and backward directions specified")]
    BothDirections,
    #[error("invalid duration: {0}")]
    InvalidDuration(#[from] humantime::DurationError),
    #[error("out of range: {0}")]
    OutOfRange(#[from] chrono::OutOfRangeError),
}

/// There are three valid formats for a biduration:
/// - "in 1h 30m" -> forward
/// - "1h 30m" -> forward
/// - "1h 30m ago" -> backward
impl FromStr for BiDuration {
    type Err = BiDurationParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split_whitespace().collect::<Vec<_>>();
        let is_explicit_forward = match parts.first() {
            Some(&"in") => true,
            Some(_) => false,
            None => return Err(BiDurationParseError::InvalidDirection(s.to_string())),
        };
        let is_backward = match parts.last() {
            Some(&"ago") => true,
            Some(_) => false,
            None => return Err(BiDurationParseError::InvalidDirection(s.to_string())),
        };

        let (direction, duration_slice) = match (is_explicit_forward, is_backward) {
            (true, true) => return Err(BiDurationParseError::BothDirections),
            (true, false) => (Direction::Forward, &parts[1..]),
            (false, true) => (Direction::Backward, &parts[..parts.len() - 1]),
            (false, false) => (Direction::Forward, &parts[..]),
        };

        let duration_str = duration_slice.to_vec().join(" ");
        let duration = humantime::parse_duration(&duration_str)?;
        let chrono_duration = Duration::from_std(duration)?;
        let chrono_duration = match direction {
            Direction::Forward => chrono_duration,
            Direction::Backward => -chrono_duration,
        };

        Ok(Self(chrono_duration))
    }
}
