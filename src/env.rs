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

use std::path::{Path, PathBuf};

use chrono_tz::Tz;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;

#[derive(Deserialize)]
pub struct Config {
    data_folder: Option<PathBuf>,
    timezone: Option<Tz>,
    #[serde(skip)]
    _data_folder: OnceCell<PathBuf>,
    #[serde(skip)]
    _timezone: OnceCell<Tz>,
}

impl Config {
    pub fn load() -> Self {
        envy::from_env().expect("Failed to load config from environment variables")
    }

    pub fn data_folder(&self) -> &Path {
        self._data_folder.get_or_init(|| {
            self.data_folder.clone().unwrap_or_else(|| dirs::data_dir().expect("Could not locate a suitable data directory. please use the DATA_FOLDER environment variable").join("punchcard"))
        })
    }

    pub fn get_output_file(&self) -> PathBuf {
        self.data_folder().join("hours.csv")
    }

    pub fn timezone(&self) -> &Tz {
        self._timezone.get_or_init(|| {
            self.timezone.unwrap_or_else(|| {
                let tz = iana_time_zone::get_timezone().expect("Could not determine local timezone. please use the TIMEZONE environment variable");
                tz.parse().expect("The timezone provided by your system could not be parsed into an IANA timezone. please use the TIMEZONE environment variable")
            })
        })
    }
}

lazy_static! {
    pub static ref CONFIG: Config = Config::load();
}
