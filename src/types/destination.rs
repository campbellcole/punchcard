use std::{
    convert::Infallible,
    fs::OpenOptions,
    io::{self, Write},
    path::PathBuf,
    str::FromStr,
};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub enum Destination {
    Stdout,
    File(PathBuf),
}

impl FromStr for Destination {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-" => Ok(Destination::Stdout),
            _ => Ok(Destination::File(PathBuf::from(s))),
        }
    }
}

impl Destination {
    pub fn to_writer(&self) -> Result<Box<dyn Write>, io::Error> {
        match self {
            Destination::Stdout => Ok(Box::new(io::stdout())),
            Destination::File(path) => Ok(Box::new(
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(path)?,
            )),
        }
    }

    pub fn is_stdout(&self) -> bool {
        matches!(self, Destination::Stdout)
    }

    pub fn unwrap_path(&self) -> &PathBuf {
        match self {
            Destination::Stdout => panic!("Cannot unwrap stdout"),
            Destination::File(path) => path,
        }
    }
}
