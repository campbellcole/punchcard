# punchcard

A simple program to keep track of your hours if you work from home.

# Compilation

Compile with the `release` feature when using release mode (nightly):

```shell
$ cargo install --no-default-features --features release
# or
$ cargo run --release --no-default-features --features release
```

If you are using stable Rust, switch `release` with `release_stable`

## Usage

```
$ punchcard --help
A CLI tool for tracking work-from-home hours.

Usage: punchcard <COMMAND>

Commands:
  in             Clock in
  out            Clock out
  report         Interpret the times and generate a report
  completions    Generate completions for the given shell
  generate-data  Generate test data
  help           Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

**NOTE: The `generate-data` command is only available with the feature flag `generate_test_data`.**
<br />
This flag is enabled by default but will be disabled if you use the [above commands](#compilation) to run/install this program.

**NOTE 2: The timezone is currently hardcoded as `America/Los_Angeles`.**
<br />
This will be fixed in a future commit, and will use either the local timezone or a timezone provided in the `.env` file.

When using the `in` or `out` commands, the `-o` option can be used to specify an offset from the current time.

Some examples of valid inputs:

- "in 1h 30m" -> add 1h 30m to the current time
- "1h 30m" -> add 1h 30m to the current time
- "1h 30m ago" -> subtract 1h 30m from the current time

The `in` prefix is optional. By default, the offset is added to the current time.

The offset is parsed by the `humantime` crate. It accepts a variety of formats. The suffixes do not have to be single letters, but they must be separated by whitespace. For example, you may use `1hours`, `1hour`, `1hr`, or `1h` to specify 1 hour.

For a list of all the suffixes, see the documentation for the `humantime` crate:

https://docs.rs/humantime/latest/humantime/fn.parse_duration.html

## Licensing

This program uses the AGPLv3 license to prevent use by corporations.
