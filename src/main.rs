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

// #[allow(dead_code)]
// fn create_road_traffic_time_logs() {
//     const PROBABILITIES: [f32; 14] = [
//         0.0, 0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0,
//     ];
//     // const STANDARD_DEVIATIONS: [f64; 11] = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
//     const STANDARD_DEVIATIONS: [f64; 14] = [
//         0.0, 0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0,
//     ];
//
//     let log = import_xes_file("./road_traffic.xes.gz", XESImportOptions::default()).unwrap();
//     let case_ids: Vec<String> = log
//         .traces
//         .iter()
//         .map(|trace| get_traceid(trace).unwrap())
//         .collect();
//
//     let rng = &mut rand::thread_rng();
//     for prob in PROBABILITIES {
//         for std in STANDARD_DEVIATIONS {
//             println!(
//                 "Creating Event Log for probability {} and std shift {}",
//                 prob, std
//             );
//             let output_path = format!("./road_traffic_newlogs/probability_{}/std_{}", prob, std);
//
//             if Path::new(format!("{output_path}/log_1.xes.gz").as_str()).exists() {
//                 println!("\talready exists... skip");
//                 continue;
//             }
//
//             let mut mutated_log = ServiceTimeStdShifter::new(std)
//                 .with_probability(prob)
//                 .for_activity("Send Fine")
//                 .apply(&log);
//
//             // Get random sample of half the caseids
//             let sample: HashSet<String> = case_ids
//                 .choose_multiple(rng, case_ids.len() / 2)
//                 .cloned()
//                 .collect();
//
//             mutated_log
//                 .traces
//                 .retain(|trace| sample.contains(&get_traceid(trace).unwrap()));
//
//             let mut non_mutated_log = log.clone();
//             non_mutated_log
//                 .traces
//                 .retain(|trace| !sample.contains(&get_traceid(trace).unwrap()));
//
//             // Save the files
//             write_xes(
//                 &non_mutated_log,
//                 format!("{}/log_1.xes.gz", output_path),
//                 true,
//             )
//             .unwrap();
//             write_xes(&mutated_log, format!("{}/log_2.xes.gz", output_path), true).unwrap();
//         }
//     }
// }

fn main() {
    let args = Args::parse();
    let res = run_cli(args);
    if let Err(e) = res {
        println!("There was an error...");
        Args::command().error(e.kind, e.message).exit();
    }
}

fn run_cli(mut args: Args) -> Result<(), CliError> {
    if args.pipeline.is_some() {
        return parse_and_execute_pipeline_file(&args);
    } else if args.input.is_none() {
        return Err(CliError::new(
            ErrorKind::MissingRequiredArgument,
            "Either an input file (--input) or a pipeline file (--pipeline) must be provided!",
        ));
    }

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

        write_xes(&new_log, args.output.unwrap(), should_compress)?
    } else {
        return Err(CliError::new(
            ErrorKind::Io,
            "The input file does not exist, or is not a file.",
        ));
    }
    Ok(())
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
