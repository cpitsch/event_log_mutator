use std::{
    fs::{create_dir_all, File},
    path::{Path, PathBuf},
};

use crate::cli::{Args, CliError};
use clap::{error::ErrorKind, CommandFactory, Parser};
use process_mining::{
    event_log::export_xes::export_xes_event_log_to_file, import_xes_file, EventLog,
    XESImportOptions,
};

use crate::{
    mutation::LogMutator,
    parsing::{parse_toml, MutationChainConfig},
};

pub mod cli;
pub mod constants;
pub mod mutation;
pub mod mutators;
pub mod parsing;
pub mod preset;
pub mod utils;

#[cfg(test)]
mod test_fixtures;

fn main() {
    let args = Args::parse();
    let res = run_cli(args);
    if let Err(e) = res {
        Args::command().error(e.kind, e.message).exit();
    }
}

fn run_cli(args: Args) -> Result<(), CliError> {
    if args.pipeline.is_some() {
        parse_and_execute_pipeline_file(&args)
    } else if args.input.is_none() {
        Err(CliError::new(
            ErrorKind::MissingRequiredArgument,
            "Either an input file (--input) or a pipeline file (--pipeline) must be provided!",
        ))
    } else {
        run_presets(args)
    }
}

pub fn parse_and_execute_pipeline_file(args: &Args) -> Result<(), CliError> {
    let path_to_pipeline = args.pipeline.clone().unwrap();
    if !path_to_pipeline.exists() {
        return Err(CliError::new(
            ErrorKind::Io,
            "The specified pipeline configuration file does not exist",
        ));
    }
    // Get the configuration from the pipeline
    let mut parsed_toml = parse_toml(&path_to_pipeline)?;
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
    if let Some(o) = args.output.clone() {
        config.output = o;
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
        Err(CliError::new(
            ErrorKind::Io,
            "The output path already exists and `--no-overwrite` specified. Aborting.",
        ))?
    }

    if input.exists() && input.is_file() {
        let log = import_xes_file(input.to_str().unwrap(), XESImportOptions::default()).unwrap();

        if args.preset.is_none() {
            return Err(CliError::new(
                ErrorKind::MissingRequiredArgument,
                "Either a pipeline (--pipeline) or a preset (--preset) must be provided!",
            ));
        }

        let mut mutation_chain = args.preset.unwrap().into_mutation_chain(&log, args.clone());

        let new_log = mutation_chain.apply(&log);

        let should_compress = args
            .output
            .as_ref()
            .unwrap()
            .extension()
            .map_or(false, |ext| ext == "gz");

        write_xes(&new_log, args.output.unwrap(), should_compress)
    } else {
        Err(CliError::new(
            ErrorKind::Io,
            "The input file does not exist, or is not a file.",
        ))
    }
}

fn write_xes(log: &EventLog, path: impl AsRef<Path>, compress: bool) -> Result<(), CliError> {
    let p: &Path = path.as_ref();
    let dir_creation_res = p.parent().map(create_dir_all);
    if dir_creation_res.is_none() || dir_creation_res.unwrap().is_err() {
        return Err(CliError::new(
            ErrorKind::Io,
            format!(
                "Something went wrong creating the directories on the path {}",
                p.to_string_lossy()
            ),
        ));
    }

    let save_res = File::create(p).map(|file| export_xes_event_log_to_file(log, file, compress));
    if save_res.is_err() || save_res.unwrap().is_err() {
        return Err(CliError::new(
            ErrorKind::Io,
            "Something went wrong while saving the file.",
        ));
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
