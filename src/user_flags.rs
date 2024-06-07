//! Contains user facing flags.
use std::path::PathBuf;

use clap::Parser;

/// Elvi is a POSIX shell written in Rust.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(flatten)]
    pub group: Group,

    /// Positional variables
    pub positionals: Option<Vec<String>>,
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub struct Group {
    /// Read commands from the command_string operand instead of from the standard input.
    ///
    /// Special parameter 0 will be set from the command_name operand
    /// and the positional parameters ($1, $2, etc.)  set
    /// from the remaining argument operands.
    #[clap(short = 'c', long = None, conflicts_with = "file")]
    pub read_from_input: Option<String>,

    /// Read from file
    #[clap(conflicts_with = "read_from_input")]
    pub file: Option<PathBuf>,
}
