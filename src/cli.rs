use crate::mutation::MutationError;
use crate::parsing::ParsingError;
use crate::preset::Preset;
use crate::utils::io::IoError;
use clap;
use clap::Parser;
use process_mining::event_log::import_xes::XESParseError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Parser, Debug, Clone)]
#[command(version)]
pub struct Args {
    /// The path to a toml file with a mutation pipeline to apply
    #[clap(long, value_name = "PATH")]
    pub pipeline: Option<PathBuf>,

    /// The path to the input XES file (.xes or .xes.gz)
    #[clap(short, long, value_name = "PATH")]
    pub input: Option<PathBuf>,

    /// The path to write the mutated log to. Defaults to path/to/input_mutated.xes
    #[clap(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// A preset mutation chain to apply
    #[clap(long, value_enum)]
    pub preset: Option<Preset>,

    /// Minimum number of supporting cases for variant. Only relevant for the
    /// --filter-variant-support preset
    #[clap(long)]
    pub support: Option<usize>,

    /// Abort if the output path already exists. Currently only works for presets.
    #[clap(long)]
    pub no_overwrite: bool,

    /// The seed to use for mutations involving randomness.
    #[clap(long)]
    pub seed: Option<u64>,

    /// Increase verbosity level. Verbosity defaults to Error. Increases following:
    /// Error, -v = Warning, -vv = Info, -vvv = Debug, -vvvv = Trace.
    #[clap(long, short='v', action=clap::ArgAction::Count, global=true)]
    pub verbose: u8,

    /// Decrease the verbosity by one level. Verbosity defaults to Error.
    #[clap(long, short, action=clap::ArgAction::Count, global=true)]
    pub quiet: u8,

    /// Validate the outputs of the pipeline against an existing output instead of
    /// writing the event logs. Elevates default verbosity to Warnings.
    #[clap(long)]
    pub validate: bool,
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
