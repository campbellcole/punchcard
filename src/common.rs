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
#![allow(non_snake_case)]

use std::path::Path;

pub const ERR_LATEST_ENTRY: &str = "Failed to get latest entry";
pub const SUGG_REPORT_ISSUE: &str =
    "If you have not manually modified this file, please report this issue";

#[inline(always)]
pub fn ERR_OPEN_CSV(p: &Path) -> String {
    format!("Failed to open or create CSV file {}", p.display())
}

#[inline(always)]
pub fn ERR_WRITE_CSV(p: &Path) -> String {
    format!("Failed to write to CSV file {}", p.display())
}

#[inline(always)]
pub fn ERR_READ_CSV(p: &Path) -> String {
    format!("Failed to read CSV file {}", p.display())
}

#[inline(always)]
pub fn SUGG_PROPER_PERMS(p: &Path) -> String {
    format!("Ensure you have proper permissions for {}", p.display())
}

pub const PRETTY_TIME: &str = "%r";
pub const PRETTY_DATE: &str = "%A, %d %B %Y";
pub const PRETTY_DATETIME: &str = "%r on %A, %d %B %Y";
pub const SLIM_DATETIME: &str = "%r %d %B %Y";

// RFC3339 with nanoseconds, no space between ns and tz
pub const CSV_DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S.%f%z";

pub const DEFAULT_CATEGORY: &str = "uncategorized";
