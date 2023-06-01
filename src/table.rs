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

// adapted from https://github.com/pola-rs/polars/blob/9a73d3c7fd53180917837280b23b33f9de251887/polars/polars-core/src/fmt.rs

use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

use comfy_table::{
    modifiers::{UTF8_ROUND_CORNERS, UTF8_SOLID_INNER_BORDERS},
    Cell, ColumnConstraint, ContentArrangement, Table, Width,
};
use polars::prelude::*;

use crate::{
    prelude::*,
    table::{color::Color, style::TableStyle},
};

use self::settings::TableSettings;

pub mod cell_alignment;
pub mod color;
pub mod settings;
pub mod style;

pub struct DataFrameDisplay<'a>(&'a DataFrame, &'a TableSettings);

impl<'a> DataFrameDisplay<'a> {
    pub fn new(df: &'a DataFrame, settings: &'a TableSettings) -> Self {
        Self(df, settings)
    }
}

fn make_str_val(v: &str, truncate: usize) -> String {
    let v_trunc = &v[..v
        .char_indices()
        .take(truncate)
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0)];
    if v == v_trunc {
        v.to_string()
    } else {
        format!("{v_trunc}…")
    }
}

fn prepare_row(
    row: Vec<Cow<'_, str>>,
    n_first: usize,
    n_last: usize,
    str_truncate: usize,
    colors: &[Color],
) -> Vec<Cell> {
    let reduce_columns = n_first + n_last < row.len();
    let mut row_str = Vec::with_capacity(n_first + n_last + reduce_columns as usize);
    for v in row[0..n_first].iter() {
        row_str.push(make_str_val(v, str_truncate));
    }
    if reduce_columns {
        row_str.push("…".to_string());
    }
    for v in row[row.len() - n_last..].iter() {
        row_str.push(make_str_val(v, str_truncate));
    }
    let it = row_str.into_iter().enumerate();
    if colors.is_empty() {
        it.map(|(_, s)| Cell::new(s)).collect()
    } else {
        it.map(|(x, s)| Cell::new(s).fg(colors[x].into())).collect()
    }
}

impl<'a> Display for DataFrameDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut df = self.0;
        // we have to have this here because we can't return a reference
        // to tdf because it's a local variable, so we need to store it
        // somewhere and hold a reference to it, and rust doesn't realize
        // the reference to ref_holder is being used
        #[allow(unused_assignments)]
        let mut ref_holder = None;
        let settings = &self.1;
        let default_colors = vec![
            Color::DarkGreen,
            Color::DarkYellow,
            Color::DarkRed,
            Color::DarkBlue,
            Color::DarkCyan,
        ];
        let column_colors = if !settings.no_color {
            settings
                .column_colors
                .as_ref()
                .cloned()
                .map(|mut c| {
                    if c.len() < default_colors.len() {
                        c.extend(default_colors[c.len()..].iter().cloned());
                    }
                    // if there are more colors, that's fine
                    c
                })
                .unwrap_or(default_colors)
        } else {
            Vec::new()
        };

        if let NumRows::Some(num_rows) = &settings.max_n_rows {
            let tdf = df.tail(Some(*num_rows));
            ref_holder = Some(tdf);
            df = ref_holder.as_ref().unwrap();
        }

        let height = df.height();
        assert!(
            df.get_columns().iter().all(|s| s.len() == height),
            "all columns must have the same length"
        );

        let max_n_cols = match settings.max_n_cols {
            NumCols::All => df.width(),
            NumCols::Some(n) => n,
        };
        let max_n_rows = match settings.max_n_rows {
            NumRows::All => height,
            NumRows::Some(n) => n,
        };

        let (n_first, n_last) = if df.width() > max_n_cols {
            ((max_n_cols + 1) / 2, max_n_cols / 2)
        } else {
            (df.width(), 0)
        };
        let reduce_columns = n_first + n_last < df.width();
        let mut names = Vec::with_capacity(n_first + n_last + reduce_columns as usize);

        let field_to_str = |f: &Field| {
            let mut name = make_str_val(f.name(), settings.string_truncate);
            let lower_bounds = name.len().clamp(5, 12);

            if settings.hide_column_names {
                name = String::new();
            }

            let column_data_type = if settings.hide_data_types {
                String::new()
            } else if settings.inline_data_types | settings.hide_column_names {
                format!("{}", f.data_type())
            } else {
                format!("\n{}", f.data_type())
            };
            let mut column_separator = "\n---";
            if settings.hide_column_separator
                | settings.hide_column_names
                | settings.hide_data_types
            {
                column_separator = "";
            }
            let s = if settings.inline_data_types & !settings.hide_data_types {
                format!("{name} ({column_data_type})")
            } else {
                format!("{name}{column_separator}{column_data_type}")
            };
            (s, lower_bounds)
        };
        let tbl_lower_bounds = |l: usize| ColumnConstraint::LowerBoundary(Width::Fixed(l as u16));

        let mut constraints = Vec::with_capacity(n_first + n_last + reduce_columns as usize);
        let fields = df.fields();

        for field in fields[0..n_first].iter() {
            let (s, l) = field_to_str(field);
            names.push(s);
            constraints.push(tbl_lower_bounds(l));
        }
        if reduce_columns {
            names.push("…".into());
            constraints.push(tbl_lower_bounds(3));
        }
        for field in fields[df.width() - n_last..].iter() {
            let (s, l) = field_to_str(field);
            names.push(s);
            constraints.push(tbl_lower_bounds(l));
        }

        let mut table = Table::new();
        table
            .load_preset(settings.style.get_style())
            .set_content_arrangement(ContentArrangement::Dynamic);

        if settings.rounded_corners && settings.style.is_utf8() {
            table.apply_modifier(UTF8_ROUND_CORNERS);
        }

        if settings.solid_inner_borders && settings.style.is_utf8() {
            table.apply_modifier(UTF8_SOLID_INNER_BORDERS);
        }

        if max_n_rows > 0 {
            if height > max_n_rows + 1 {
                let mut rows = Vec::with_capacity(std::cmp::max(max_n_rows, 2));
                for i in 0..std::cmp::max(max_n_rows / 2, 1) {
                    let row = df
                        .get_columns()
                        .iter()
                        .map(|s| s.str_value(i).unwrap())
                        .collect();
                    rows.push(prepare_row(
                        row,
                        n_first,
                        n_last,
                        settings.string_truncate,
                        &column_colors,
                    ));
                }
                let dots = rows[0].iter().map(|_| Cell::new("…")).collect();
                rows.push(dots);
                if max_n_rows > 1 {
                    for i in (height - (max_n_rows + 1) / 2)..height {
                        let row = df
                            .get_columns()
                            .iter()
                            .map(|s| s.str_value(i).unwrap())
                            .collect();
                        rows.push(prepare_row(
                            row,
                            n_first,
                            n_last,
                            settings.string_truncate,
                            &column_colors,
                        ));
                    }
                }
                table.add_rows(rows);
            } else {
                for i in 0..height {
                    if df.width() > 0 {
                        let row = df
                            .get_columns()
                            .iter()
                            .map(|s| s.str_value(i).unwrap())
                            .collect();
                        table.add_row(prepare_row(
                            row,
                            n_first,
                            n_last,
                            settings.string_truncate,
                            &column_colors,
                        ));
                    } else {
                        break;
                    }
                }
            }
        } else if height > 0 {
            let dots: Vec<String> = df.get_columns().iter().map(|_| "…".to_string()).collect();
            table.add_row(dots);
        }

        if !(settings.hide_column_names && settings.hide_data_types) {
            let it = names.into_iter();

            let names = if settings.no_color {
                it.map(Cell::new).collect::<Vec<_>>()
            } else {
                it.map(|s| Cell::new(s).fg(settings.header_color.into()))
                    .collect::<Vec<_>>()
            };

            table.set_header(names).set_constraints(constraints);
        }

        if matches!(settings.style, TableStyle::AsciiMarkdown) {
            table.set_width(u16::MAX);
        } else if let Some(w) = settings.width {
            table.set_width(w);
        } else if !table.is_tty() {
            table.set_width(100);
        }

        for column in table.column_iter_mut() {
            column.set_cell_alignment(settings.cell_alignment.get());
        }

        write!(f, "{table}")?;

        Ok(())
    }
}
