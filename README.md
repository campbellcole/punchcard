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
Usage: punchcard [OPTIONS] <COMMAND>

Commands:
  in    Clock in
  out   Clock out
  help  Print this message or the help of the given subcommand(s)

Options:
  -o, --offset-from-now <OFFSET_FROM_NOW>
          The offset from the current time to use as the clock in/out time
  -h, --help
          Print help
  -V, --version
          Print version
```

The `-o` option is used to specify an offset from the current time.

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
