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

use thiserror::Error;

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone, Copy)]
pub enum Quantity {
    All,
    Some(usize),
}

impl ToString for Quantity {
    fn to_string(&self) -> String {
        match self {
            Quantity::All => "all".into(),
            Quantity::Some(num) => num.to_string(),
        }
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Error)]
pub enum QuantityError {
    #[error("Quantity cannot be zero")]
    Zero,
    #[error("Unknown value. Must be a positive integer or \"all\"")]
    Unknown,
}

impl FromStr for Quantity {
    type Err = QuantityError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<usize>() {
            Ok(num) if num == 0 => Err(QuantityError::Zero),
            Ok(num) => Ok(Quantity::Some(num)),
            Err(_) if s.eq_ignore_ascii_case("all") => Ok(Quantity::All),
            Err(_) => Err(QuantityError::Unknown),
        }
    }
}

pub type NumRows = Quantity;
pub type NumCols = Quantity;
