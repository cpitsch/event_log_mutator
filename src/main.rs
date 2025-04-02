use std::path::{Path, PathBuf};

use clap::Parser;
use cli::{CliError, CliResult, Mode};
use colored::Colorize;
use parsing::mutation_value::MutationValue;
use utils::io::IoError;

use crate::{cli::Args, parsing::MutationChainConfig, utils::logging::init_logger};

pub mod cli;
pub mod constants;
pub mod mutation;
pub mod mutators;
pub mod parsing;
pub mod preset;
pub mod utils;

#[cfg(test)]
mod test_fixtures;

fn main() -> ! {
    let mut args = Args::parse();
    args.verbose = args
        .verbose
        .saturating_add_signed(args.mode.relative_logging_level());
    init_logger(args.verbose, args.quiet);
    let res = run_cli(args);
    if let Err(e) = res {
        eprintln!("{}: {e}", "error".red().bold());
        std::process::exit(2);
    }
    std::process::exit(0);
}

fn run_cli(args: Args) -> Result<(), CliError> {
    match args.mode {
        Mode::Preset {
            no_overwrite,
            preset,
        } => {
            if args.input.is_none() {
                Err(CliError::MissingRequiredArgument(
                    "Missing input file (--input) to apply the preset to!",
                ))
            } else {
                preset.execute(args, no_overwrite)
            }
        }
        Mode::Pipeline { ref path, validate } => {
            parse_and_execute_pipeline_file(&args, path, validate)
        }
    }
}

pub fn parse_and_execute_pipeline_file(
    args: &Args,
    pipeline_path: impl AsRef<Path>,
    validate: bool,
) -> Result<(), CliError> {
    // Get the configuration from the pipeline
    let mut parsed_toml = MutationChainConfig::parse_file(pipeline_path)?;
    parsed_toml = overwrite_pipeline_config_with_cli_args(args, parsed_toml);
    parsed_toml = resolve_tilde_in_paths(parsed_toml)?;

    if validate {
        parsed_toml.validate()
    } else {
        parsed_toml.execute()
    }
}

pub fn overwrite_pipeline_config_with_cli_args(
    args: &Args,
    mut config: MutationChainConfig,
) -> MutationChainConfig {
    // If an input file is explicitly specified, override pipeline config with that
    if let Some(input) = &args.input {
        config.input.clone_from(input);
    }
    // If an output dir is explicitly specified, override pipeline config with that
    if args.output.is_some() {
        config.output = args.output.clone();
    }

    // If a seed is explicitly specified, override pipeline config with that
    if args.seed.is_some() {
        config.pipeline.seed = args.seed.map(MutationValue::Value);
    }

    config
}

/// In all PathBuf fields of the configuration, replace ~ with the home directory
/// so that using ~ is supported
pub fn resolve_tilde_in_paths(mut config: MutationChainConfig) -> CliResult<MutationChainConfig> {
    config.input = resolve_tilde(config.input)?;
    if let Some(output) = config.output {
        config.output = Some(resolve_tilde(output)?)
    }
    Ok(config)
}

/// If the PathBuf contains "~", replace it with the home directory
fn resolve_tilde(path: PathBuf) -> CliResult<PathBuf> {
    if !path.starts_with("~") {
        Ok(path)
    } else {
        let mut home_dir = dirs::home_dir().ok_or(IoError::NoHomeDirectory)?;
        let stripped = path.strip_prefix("~").unwrap();
        home_dir.push(stripped);
        Ok(home_dir)
    }
}
