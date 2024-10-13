use clap::Parser;
use cli::CliError;
use colored::Colorize;
use preset::Preset;

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
    let args = Args::parse();
    init_logger(args.verbose);
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
        Preset::execute(args)
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
