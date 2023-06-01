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
    io::{Read, Write},
    process::{Command, Stdio},
};

use polars::prelude::{DataFrame, LazyFrame};

use crate::{
    prelude::*,
    table::{color::Color, settings::TableSettings, style::TableStyle, DataFrameDisplay},
};

use super::{daily, weekly, ReportSettings, ReportType, COL_DURATION};

const MARKDOWN_TEMPLATE: &str = include_str!("../../../web/template.md");

pub fn generate_copyable_report(lf: LazyFrame, settings: &ReportSettings) -> Result<()> {
    let mut table = String::new();

    let table_settings = TableSettings {
        style: TableStyle::AsciiMarkdown,
        no_color: true,
        ..settings.table_settings.clone()
    };

    let prepped = match settings.report_type {
        ReportType::Daily => daily::prepare_for_display(lf.clone(), settings),
        ReportType::Weekly => weekly::prepare_for_display(lf.clone(), settings),
    };

    let df = prepped.collect()?;

    let display = DataFrameDisplay::new(&df, &table_settings);

    {
        use std::fmt::Write;

        write!(table, "{}", display)?;
    }

    let mut template = String::from(MARKDOWN_TEMPLATE);

    template = template.replace(
        "%%REPORT_DATE%%",
        &Local::now().format("%Y-%m-%d").to_string(),
    );

    template = template.replace("%%REPORT_TABLE%%", &table);

    // this table retains original data types so we can use it to calculate the total hours
    let df = lf.collect()?;

    let total_hours = df.column("Total Hours").unwrap().sum::<i64>().unwrap();
    let total_hours = chrono::Duration::nanoseconds(total_hours);
    let total_hours = BiDuration::new(total_hours);
    let total_hours_str = total_hours.to_friendly_hours_string();

    template = template.replace("%%TOTAL_HOURS%%", &total_hours_str);

    let mut pandoc = Command::new("pandoc");
    pandoc.stdin(Stdio::piped()).stdout(Stdio::piped());

    let mut pandoc = pandoc.spawn()?;

    let mut stdin = pandoc.stdin.take().unwrap();
    let mut stdout = pandoc.stdout.take().unwrap();

    stdin.write_all(template.as_bytes())?;
    stdin.flush()?;
    drop(stdin);

    let mut html = String::new();

    stdout.read_to_string(&mut html)?;
    drop(stdout);

    Ok(())
}
