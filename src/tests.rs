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

use chrono::Duration;

use crate::quantity::{Quantity, QuantityError};

use super::biduration::{BiDuration, BiDurationParseError};

#[test]
fn test_parse_biduration() {
    let expected_duration = Duration::hours(5) + Duration::minutes(2) + Duration::seconds(3);
    let cases = [
        ("5h 2m 3s", Ok(BiDuration::new(expected_duration))),
        ("in 5h 2m 3s", Ok(BiDuration::new(expected_duration))),
        ("5h 2m 3s ago", Ok(BiDuration::new(-expected_duration))),
        ("in 5h 2m 3s ago", Err(BiDurationParseError::BothDirections)),
    ];

    for (input, output) in cases {
        assert_eq!(input.parse::<BiDuration>(), output);
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

    for input in cases {
        assert_eq!(
            input.0.parse::<BiDuration>().unwrap().to_friendly_string(),
            input.1
        );
    }
}

#[test]
fn test_parse_num_rows() {
    let cases = [
        ("all", Ok(Quantity::All)),
        ("0", Err(QuantityError::Zero)),
        ("50", Ok(Quantity::Some(50))),
    ];

    for (input, output) in cases {
        assert_eq!(input.parse::<Quantity>(), output);
    }
}
