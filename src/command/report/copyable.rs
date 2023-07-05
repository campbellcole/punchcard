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
    fs::File,
    io::{Read, Write},
    process::{Command, Stdio},
};

use polars::prelude::LazyFrame;
use snailquote::escape;

use crate::{
    prelude::*,
    table::{settings::TableSettings, style::TableStyle, DataFrameDisplay},
};

use super::{daily, weekly, ReportSettings, ReportType};

const MARKDOWN_TEMPLATE: &str = include_str!("../../../web/template.md");
const HTML_TEMPLATE: &str = include_str!("../../../web/template.html");

const REPORT_DATE_PLACEHOLDER: &str = "%%REPORT_DATE%%";
const REPORT_TABLE_PLACEHOLDER: &str = "%%REPORT_TABLE%%";
const TOTAL_HOURS_PLACEHOLDER: &str = "%%TOTAL_HOURS%%";
const REPORT_HTML_PLACEHOLDER: &str = "%%REPORT_HTML%%";

pub fn generate_copyable_report(lf: LazyFrame, settings: &ReportSettings) -> Result<()> {
    let mut table = String::new();

    let table_settings = TableSettings {
        style: TableStyle::AsciiMarkdown,
        no_color: true,
        ..settings.table_settings.clone()
    };

    let prepped = match settings.report_type.as_ref().cloned().unwrap_or_default() {
        ReportType::Daily => daily::prepare_for_display(lf.clone(), settings),
        ReportType::Weekly(_) => weekly::prepare_for_display(lf.clone(), settings),
    };

    let df = prepped.collect()?;

    let display = DataFrameDisplay::new(&df, &table_settings);

    {
        use std::fmt::Write;

        write!(table, "{}", display)?;
    }

    let mut template = String::from(MARKDOWN_TEMPLATE);

    template = template.replace(
        REPORT_DATE_PLACEHOLDER,
        &Local::now().format("%Y-%m-%d").to_string(),
    );

    template = template.replace(REPORT_TABLE_PLACEHOLDER, &table);

    // this table retains original data types so we can use it to calculate the total hours
    let df = lf.collect()?;

    let total_hours = df.column("Total Hours").unwrap().sum::<i64>().unwrap();
    let total_hours = chrono::Duration::nanoseconds(total_hours);
    let total_hours = BiDuration::new(total_hours);
    let total_hours_str = total_hours.to_friendly_hours_string();

    template = template.replace(TOTAL_HOURS_PLACEHOLDER, &total_hours_str);

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

    html = escape(&html).to_string();

    let full_html = HTML_TEMPLATE.replace(REPORT_HTML_PLACEHOLDER, &html);

    let temp_dir = temp_dir::TempDir::new()?;

    let temp_file_path = temp_dir.path().join("report.html");

    let mut temp_file = File::create(&temp_file_path)?;

    write!(temp_file, "{}", full_html)?;
    temp_file.flush()?;

    println!("Opening report in browser...");
    println!("Follow instructions on the page that opens.");
    std::thread::sleep(std::time::Duration::from_secs(3));

    let mut chromium = Command::new("chromium");

    chromium.stdout(Stdio::null()).stderr(Stdio::null());

    chromium.arg(&temp_file_path);

    chromium.spawn()?.wait()?;

    println!("Report closed.");

    Ok(())
}
