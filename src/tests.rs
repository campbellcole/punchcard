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

use super::biduration::{BiDuration, BiDurationParseError};

#[test]
fn test_parse_biduration() {
    let inputs = ["5h 2m 3s", "in 5h 2m 3s", "5h 2m 3s ago", "in 5h 2m 3s ago"];

    let expected_duration = Duration::hours(5) + Duration::minutes(2) + Duration::seconds(3);

    let outputs = [
        Ok(BiDuration(expected_duration)),
        Ok(BiDuration(expected_duration)),
        Ok(BiDuration(-expected_duration)),
        Err(BiDurationParseError::BothDirections),
    ];

    for (input, output) in inputs.iter().zip(outputs.iter()) {
        assert_eq!(input.parse::<BiDuration>(), *output);
    }
}
