use std::path::{Path, PathBuf};

use clap::Parser;
use cli::CliError;
use colored::Colorize;
use process_mining::{import_xes_file, XESImportOptions};
use utils::io::{write_xes, IoError};

use crate::{cli::Args, mutation::LogMutator, parsing::MutationChainConfig};

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
    let args = Args::parse();
    let res = run_cli(args);
    if let Err(e) = res {
        eprintln!("{}: {e}", "error".red().bold());
        std::process::exit(2);
    }
    std::process::exit(0);
}

fn run_cli(args: Args) -> Result<(), CliError> {
    if args.pipeline.is_some() {
        parse_and_execute_pipeline_file(&args)
    } else if args.input.is_none() {
        Err(CliError::MissingRequiredArgument(
            "Either an input file (--input) or a pipeline file (--pipeline) must be provided!",
        ))
    } else {
        run_presets(args)
    }
}

pub fn parse_and_execute_pipeline_file(args: &Args) -> Result<(), CliError> {
    let path_to_pipeline = args.pipeline.clone().unwrap();
    // Get the configuration from the pipeline
    let mut parsed_toml = MutationChainConfig::parse_file(&path_to_pipeline)?;
    parsed_toml = overwrite_pipeline_config_with_cli_args(args, parsed_toml);

    parsed_toml.execute()
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
        config.seed = args.seed;
    }

    config
}

fn run_presets(mut args: Args) -> Result<(), CliError> {
    let input = args.input.as_ref().unwrap();
    if args.output.is_none() {
        args.output = Some(get_output_path(input));
    }

    if args.no_overwrite && args.output.clone().unwrap().exists() {
        Err(IoError::FileExists(args.clone().output.unwrap()))?
    }

    if input.exists() && input.is_file() {
        let log = import_xes_file(input.to_str().unwrap(), XESImportOptions::default()).unwrap();

        if args.preset.is_none() {
            return Err(CliError::MissingRequiredArgument(
                "Either a pipeline (--pipeline) or a preset (--preset) must be provided!",
            ));
        }

        let mut mutation_chain = args.preset.unwrap().into_mutation_chain(&log, args.clone());

        let new_log = mutation_chain.apply(&log)?;

        let should_compress = args
            .output
            .as_ref()
            .unwrap()
            .extension()
            .map_or(false, |ext| ext == "gz");

        write_xes(&new_log, args.output.unwrap(), should_compress)?;
    } else {
        Err(IoError::FileNotFound(input.clone()))?
    }
    Ok(())
}

fn get_output_path(input_path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    if let Some(parent) = input_path.parent() {
        out.push(parent);
    }

    // Prepend 'mutated_' and call it a day
    let name_string = input_path
        .file_name()
        .expect("The path should be a file.")
        .to_string_lossy();
    out.push(format!("mutated_{}", name_string));
    out
}
