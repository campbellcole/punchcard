# punchcard

A simple program to keep track of your hours if you work from home.

![clock in stdout](./assets/clock_in.png)
![clock status image](./assets/status.png)
![example report table](./assets/report.png)
<small>Colors are customizable through arguments on the `report` command. Installing completions is highly recommended to customize the table as there are a lot of options.</small>

## Installation

**By default, the program requires nightly to support SIMD.**

```shell
$ cargo install --git https://github.com/campbellcole/punchcard --branch main
# or
$ git clone https://github.com/campbellcole/punchcard && cd punchcard
$ cargo run --release
```

If you are using stable Rust, compile with `--no-default-features --features stable`

#### Completions

Print the completions file with `punchcard completions <your shell>` and pipe it to the appropriate folder for your shell.

### Development/Debug builds

```shell
$ cargo run --no-default-features --features debug ...
```

## Usage

```
$ punchcard --help
A CLI tool for tracking work-from-home hours.

Usage: punchcard [OPTIONS] <COMMAND>

Commands:
  in             Clock in
  out            Clock out
  toggle         Clock either in or out
  status         Check the current status
  report         Interpret the times and generate a report
  completions    Generate completions for the given shell
  generate-data  Generate test data
  help           Print this message or the help of the given subcommand(s)

Options:
  -d, --data-folder <DATA_FOLDER>  [env: PUNCHCARD_DATA_FOLDER=.] [default: /home/campbell/.local/share/punchcard]
  -t, --timezone <TIMEZONE>        [env: PUNCHCARD_TIMEZONE=] [default: America/Los_Angeles]
  -h, --help                       Print help
  -V, --version                    Print version
```

**NOTE: The `generate-data` subcommand is only available with the feature flag `generate_test_data`.**
<br />
This flag is enabled by the `debug` feature flag, but can be enabled in release builds as well.

When using the `in`, `out`, `toggle`, and `status` subcommands, the `-o` option can be used to specify an offset from the current time.

Some examples of valid inputs:

- "in 1h 30m" -> add 1h 30m to the current time
- "1h 30m" -> add 1h 30m to the current time
- "1h 30m ago" -> subtract 1h 30m from the current time

The `in` prefix is optional; by default, the offset is added to the current time.

The offset is parsed by the `humantime` crate. It accepts a variety of formats. The suffixes do not have to be single letters, but they must be separated by whitespace. For example, you may use `1hours`, `1hour`, `1hr`, or `1h` to specify 1 hour.

For a list of all the suffixes, see the documentation for the `humantime` crate:

https://docs.rs/humantime/latest/humantime/fn.parse_duration.html
