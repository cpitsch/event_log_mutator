use crate::preset::Preset;
use clap;
use clap::error::ErrorKind;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
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

    /// If present, and no preset is selected, apply mutations to the event log.
    /// Otherwise, only apply bootstrapping
    #[clap(long)]
    pub mutate: bool,

    /// A preset mutation chain to apply
    #[clap(long, value_enum)]
    pub preset: Option<Preset>,

    /// Minimum number of supporting cases for variant. Only relevant for
    /// --filter-variant-support
    #[clap(long)]
    pub support: Option<usize>,

    /// Factor to multiply service time with when using road-traffic preset.
    #[clap(long)]
    pub severity: Option<f32>,

    /// Probability to apply mutation. Only used in road-traffic preset.
    #[clap(long)]
    pub probability: Option<f32>,

    /// Abort if the output path already exists.
    #[clap(long)]
    pub no_overwrite: bool,

    /// The seed to use for mutations involving randomness.
    #[clap(long)]
    pub seed: Option<u64>,
}

#[derive(Debug)]
pub struct CliError {
    pub kind: ErrorKind,
    pub message: String,
}

impl CliError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}
