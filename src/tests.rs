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

use std::path::PathBuf;

use chrono::Duration;

use crate::types::{
    BiDuration, BiDurationParseError, Destination, Month, ParseMonthError, Quantity, QuantityError,
};

#[test]
fn test_parse_biduration() {
    let expected_duration = Duration::hours(5) + Duration::minutes(2) + Duration::seconds(3);
    let cases = [
        ("5h 2m 3s", Ok(BiDuration::new(expected_duration))),
        ("in 5h 2m 3s", Ok(BiDuration::new(expected_duration))),
        ("5h 2m 3s ago", Ok(BiDuration::new(-expected_duration))),
        ("in 5h 2m 3s ago", Err(BiDurationParseError::BothDirections)),
    ];

    for (input, expected) in cases {
        assert_eq!(input.parse::<BiDuration>(), expected);
    }
}

#[test]
fn test_format_biduration() {
    // the output format always contains `in` or `ago`
    // and it also expands some abbreviations
    let cases = [
        ("24d 12h 6m 3s", "in 24days 12h 6m 3s"),
        ("24d 12h 6m 3s", "in 24days 12h 6m 3s"),
        ("24d 12h 6m 3s ago", "24days 12h 6m 3s ago"),
    ];

    for (input, expected) in cases {
        assert_eq!(
            input.parse::<BiDuration>().unwrap().to_friendly_string(),
            expected
        );
    }
}

#[test]
fn test_format_biduration_hours() {
    let cases = [
        (
            BiDuration::new(Duration::nanoseconds(i64::MAX)),
            "2562047 hours 47 minutes",
        ),
        (BiDuration::new(Duration::minutes(100)), "1 hour 40 minutes"),
        // negative durations are swallowed because we only care about magnitude
        (BiDuration::new(Duration::minutes(-120)), "2 hours"),
        (
            BiDuration::new(Duration::nanoseconds(i64::MIN)),
            "2562047 hours 47 minutes",
        ),
        (BiDuration::new(Duration::seconds(29)), "0 minutes"),
        (BiDuration::new(Duration::seconds(30)), "1 minute"),
        (BiDuration::new(Duration::seconds(0)), "0 minutes"),
    ];

    for (input, expected) in cases {
        assert_eq!(input.to_friendly_hours_string(), expected);
    }
}

#[test]
fn test_parse_num_rows() {
    let cases = [
        ("all", Ok(Quantity::All)),
        ("0", Err(QuantityError::Zero)),
        ("50", Ok(Quantity::Some(50))),
    ];

    for (input, expected) in cases {
        assert_eq!(input.parse::<Quantity>(), expected);
    }
}

#[test]
fn test_parse_destination() {
    let cases = [
        (
            "/some/random/path",
            Destination::File(PathBuf::from("/some/random/path")),
        ),
        ("-", Destination::Stdout),
    ];

    for (input, expected) in cases {
        assert_eq!(input.parse::<Destination>(), Ok(expected));
    }
}

#[test]
fn test_parse_month() {
    let cases = [
        ("all", Ok(Month::All)),
        ("2", Ok(Month::February)),
        ("AugUST", Ok(Month::August)),
        ("99", Err(ParseMonthError::InvalidMonthNumber(99))),
        ("foo", Err(ParseMonthError::UnknownMonth("foo".to_string()))),
    ];

    for (input, expected) in cases {
        assert_eq!(input.parse::<Month>(), expected);
    }
}
