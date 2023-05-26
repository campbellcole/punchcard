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
    fmt::Write,
    ops::{Add, Deref},
    str::FromStr,
};

use chrono::{DateTime, Duration, OutOfRangeError, TimeZone};
use thiserror::Error;

/// A wrapper around the `humantime` crate which allows parsing negative durations.
///
/// The `humantime` crate only allows parsing `std::time::Duration`s which are positive.
/// This wrapper uses that parser to first grab a `std::time::Duration` and then
/// converts that into a `chrono::Duration` which can be negative.
///
/// Accepts the following formats:
/// - "in 1h 30m" -> forward
/// - "1h 30m" -> forward
/// - "1h 30m ago" -> backward
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BiDuration(pub(crate) Duration);

fn item(s: &mut String, is_first: &mut bool, name: &str, value: u64) {
    if value > 0 {
        if !*is_first {
            s.push(' ');
        }
        *is_first = false;
        write!(s, "{} {}", value, name).unwrap();
        if value > 1 {
            s.push('s');
        }
    }
}

impl BiDuration {
    pub fn to_std_duration(&self) -> (std::time::Duration, Direction) {
        let duration = self.0;
        let (positive_duration, direction) = if **self < Duration::zero() {
            (-duration, Direction::Backward)
        } else {
            (duration, Direction::Forward)
        };
        // SAFETY: cannot fail because we've inverted negative durations
        // and i64::MAX < u64::MAX
        let std_duration = positive_duration.to_std().unwrap();
        (std_duration, direction)
    }

    pub fn to_friendly_string(&self) -> String {
        let (std_duration, direction) = self.to_std_duration();
        let duration_str = humantime::format_duration(std_duration).to_string();
        match direction {
            Direction::Forward => format!("in {}", duration_str),
            Direction::Backward => format!("{} ago", duration_str),
        }
    }

    pub fn to_friendly_hours_string(&self) -> String {
        let (std_duration, _) = self.to_std_duration();

        let secs = std_duration.as_secs();

        if secs == 0 {
            return "0 minutes".into();
        }

        // Round up to the nearest minute
        let rounded_minutes = ((secs % 60) as f64 / 60.0).round() as u64;
        // Calculate the total number of minutes in the duration, rounded
        let minutes = secs / 60 + rounded_minutes;
        // Calculate how many hours were in those minutes
        let hours = minutes / 60;
        // Remove the hours from the minutes so we're left with just hours and minutes
        let minutes = minutes % 60;

        let mut s = String::new();
        let is_first = &mut true;
        item(&mut s, is_first, "hour", hours);
        item(&mut s, is_first, "minute", minutes);

        if s.is_empty() {
            s.push_str("0 minutes");
        }

        s
    }

    /// Convert a `std::time::Duration` and a direction into a `BiDuration`.
    pub fn new_std(
        duration: std::time::Duration,
        direction: Direction,
    ) -> Result<Self, OutOfRangeError> {
        let chrono_duration = Duration::from_std(duration)?;
        let chrono_duration = match direction {
            Direction::Forward => chrono_duration,
            Direction::Backward => -chrono_duration,
        };
        Ok(Self(chrono_duration))
    }

    /// Convert a `time::duration::Duration` into a `BiDuration`.
    pub const fn new(duration: Duration) -> Self {
        Self(duration)
    }
}

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
    #[error("Invalid direction: {0}")]
    InvalidDirection(String),
    #[error("Both forward and backward directions specified")]
    BothDirections,
    #[error("Invalid duration: {0}")]
    InvalidDuration(#[from] humantime::DurationError),
    #[error("Out of range: {0}")]
    OutOfRange(#[from] chrono::OutOfRangeError),
}

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
            // in .. ago
            (true, true) => return Err(BiDurationParseError::BothDirections),
            // in ..
            (true, false) => (Direction::Forward, &parts[1..]),
            // .. ago
            (false, true) => (Direction::Backward, &parts[..parts.len() - 1]),
            // ..
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
