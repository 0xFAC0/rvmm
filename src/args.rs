use core::fmt;

use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(name="Fuck Vanguard")]
#[command(version="0.1")]
#[command(about="We try to fuck Vanguard", long_about = None)]
pub struct Cli {
    #[arg(short, long, default_value_t=Verbosity::Debug, value_enum)]
    pub verbosity: Verbosity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Verbosity {
    Debug,
    Info,
    Warn,
    Error
}

impl fmt::Display for Verbosity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
