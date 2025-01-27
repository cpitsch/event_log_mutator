use crate::mutation::MutationError;
use crate::parsing::ParsingError;
use crate::preset::Preset;
use crate::utils::io::IoError;
use clap::Parser;
use clap::{self, Subcommand};
use process_mining::event_log::import_xes::XESParseError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Parser, Debug, Clone)]
#[command(
    version,
    subcommand_value_name = "MODE",
    subcommand_help_heading = "Modes"
)]
pub struct Args {
    #[command(subcommand)]
    pub mode: Mode,

    /// The path to the input XES file (.xes or .xes.gz)
    #[clap(short, long, value_name = "PATH", global = true)]
    pub input: Option<PathBuf>,

    /// The path to write the mutated log to. Defaults to path/to/input_mutated.xes
    #[clap(short, long, value_name = "PATH", global = true)]
    pub output: Option<PathBuf>,

    /// The seed to use for mutations involving randomness.
    #[clap(long, global = true)]
    pub seed: Option<u64>,

    /// Increase verbosity level. Verbosity defaults to Error. Increases following:
    /// Error, -v = Warning, -vv = Info, -vvv = Debug, -vvvv = Trace.
    #[clap(long, short='v', action=clap::ArgAction::Count, global=true)]
    pub verbose: u8,

    /// Decrease the verbosity by one level. Verbosity defaults to Error.
    #[clap(long, short, action=clap::ArgAction::Count, global=true)]
    pub quiet: u8,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Mode {
    /// Apply a mutation pipeline from a TOML file.
    Pipeline {
        /// The path to a toml file with a mutation pipeline to apply
        path: PathBuf,
        #[clap(long)]
        /// Validate the outputs of the pipeline against an existing output instead of
        /// writing the event logs. Elevates default verbosity to Warnings.
        #[clap(long)]
        validate: bool,
    },
    /// Apply a preset mutation.
    #[command(subcommand_value_name = "PRESET", subcommand_help_heading = "Presets")]
    Preset {
        #[command(subcommand)]
        preset: Preset,
        /// Abort if the output path already exists.
        #[clap(long)]
        no_overwrite: bool,
    },
}

impl Mode {
    /// How much to increase the default logging level by.
    pub fn relative_logging_level(&self) -> i8 {
        match self {
            // Default verbosity should be `warn` (one level higher than usual default.
            Mode::Pipeline { validate: true, .. } => 1,
            _ => 0,
        }
    }
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error(transparent)]
    IoError(#[from] IoError),
    #[error(transparent)]
    MutationError(#[from] MutationError),
    #[error(transparent)]
    ParsingError(#[from] ParsingError),
    #[error("{0}")]
    MissingRequiredArgument(&'static str),
    #[error(transparent)]
    XESParseError(#[from] XESParseError),
}

pub type CliResult<T> = Result<T, CliError>;
