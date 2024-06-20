use std::{
    collections::HashSet,
    fs::{create_dir_all, File},
    path::{Path, PathBuf},
};

use clap::{error::ErrorKind, CommandFactory, Parser, ValueEnum};
use mutators::{
    filters::VariantSupportFilter, ActivityRemover, EventSwapper, LogBootstrapper,
    PartialOrderCreator, ServiceTimeMultiplier,
};
use process_mining::{
    event_log::export_xes::export_xes_event_log_to_file, import_xes_file, EventLog,
    XESImportOptions,
};
use rand::seq::SliceRandom;

use crate::{
    mutation::{LogMutator, MutationChain},
    mutators::ServiceTimeStdShifter,
    parsing::{
        mutation_config_vec_to_path, parametrized_pipeline::get_parametrized_pipeline_output_root,
        parse_toml, PipelineConfig,
    },
    utils::get_traceid,
};

pub mod constants;
pub mod mutation;
pub mod mutators;
pub mod parsing;
pub mod utils;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    /// The path to the input XES file (.xes or .xes.gz)
    #[clap(short, long, value_name = "PATH")]
    input: Option<PathBuf>,

    /// The path to a toml file with a mutation pipeline to apply
    #[clap(long, value_name = "PATH")]
    pipeline: Option<PathBuf>,

    /// The path to write the mutated log to. Defaults to /path/to/input_mutated.xes
    #[clap(short, long, value_name = "PATH")]
    output: Option<PathBuf>,

    /// If present, and no preset is selected, apply mutations to the event log. Otherwise, only
    /// apply bootstrapping
    #[clap(long)]
    mutate: bool,

    /// A preset mutation chain to apply
    #[clap(long, value_enum)]
    preset: Option<Preset>,

    /// Minimum number of supporting cases for variant. Only relevant for --filter-variant-support
    #[clap(long)]
    support: Option<usize>,

    /// Factor to multiply service time with when using road-traffic preset.
    #[clap(long)]
    severity: Option<f32>,

    /// Probability to apply mutation. Only used in road-traffic preset.
    #[clap(long)]
    probability: Option<f32>,

    /// Abort if the output path already exists.
    #[clap(long)]
    no_overwrite: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Preset {
    /// Bootstrap a "new" event log of the same size by sampling cases with replacement
    Bootstrap,
    /// Turn an atomic event log into a partially ordered event log by using the
    /// time since the previous event as the service time
    PartialOrder,
    Bpi12OnlyServiceTime,
    Bpi12,
    /// Bootstrap, then multiply the service time of "Send Fine" by 2
    RoadTraffic,
    /// Bootstrap, then swap events "Send Fine" and "Payment"
    RoadTrafficSwap,
    /// Retain only the cases whose variant is supported by at least `n` cases total
    FilterVariantSupport,
}

#[derive(Debug)]
pub struct CliError {
    kind: ErrorKind,
    message: String,
}

impl CliError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl Preset {
    pub fn into_mutation_chain(self, log: &EventLog, args: Args) -> MutationChain {
        match self {
            Self::Bootstrap => {
                MutationChain::new().with_mutation(LogBootstrapper::new(log.traces.len()))
            }
            Self::PartialOrder => MutationChain::new().with_mutation(PartialOrderCreator::new()),
            Self::Bpi12 => MutationChain::new()
                .with_mutation(LogBootstrapper::new(log.traces.len()))
                .with_mutation(
                    ServiceTimeMultiplier::new(2.0)
                        .for_activity("W_Completeren aanvraag")
                        .with_probability(1.0),
                )
                .with_mutation(
                    // Only 270 instances in the original log
                    ActivityRemover::new("W_Beoordelen fraude").with_probability(1.0),
                ),
            Self::Bpi12OnlyServiceTime => MutationChain::new()
                .with_mutation(LogBootstrapper::new(log.traces.len()))
                .with_mutation(
                    ServiceTimeMultiplier::new(2.0)
                        .for_activity("W_Completeren aanvraag")
                        .with_probability(1.0),
                ),
            Self::RoadTraffic => MutationChain::new()
                .with_mutation(LogBootstrapper::new(log.traces.len()))
                .with_mutation(
                    ServiceTimeMultiplier::new(args.severity.unwrap_or(2.0))
                        .for_activity("Send Fine")
                        .with_probability(args.probability.unwrap_or(1.0)),
                ),
            // .with_mutation(
            //     ServiceTimeMultiplier::new(2.0)
            //         .for_activity("Send for Credit Collection"),
            // );
            Self::RoadTrafficSwap => MutationChain::new()
                .with_mutation(LogBootstrapper::new(log.traces.len()))
                .with_mutation(EventSwapper::new("Send Fine", "Payment")),
            Self::FilterVariantSupport => {
                MutationChain::new()
                    .with_mutation(VariantSupportFilter::new(args.support.expect(
                        "Variant Support Filter requires the `--support` flag to be set.",
                    )))
            }
        }
    }
}

pub fn parse_and_execute_pipeline_file(args: &Args) -> Result<(), CliError> {
    let path_to_pipeline = args.pipeline.clone().unwrap();
    if !path_to_pipeline.exists() {
        return Err(CliError::new(
            ErrorKind::Io,
            "The specified pipeline configuration file does not exist".to_string(),
        ));
    }
    // Get the configuration from the pipeline
    let mut parsed_toml = parse_toml(&path_to_pipeline)?;

    // If an input file is explicitly specified, override pipeline config with that
    if let Some(input) = &args.input {
        parsed_toml.input.clone_from(input);
    }
    // If an output dir is explicitly specified, override pipeline config with that
    if let Some(output) = &args.output {
        parsed_toml.output = Some(output.clone());
    }
    // Read the event log
    let log = import_xes_file(
        &parsed_toml.clone().input.to_string_lossy(),
        XESImportOptions::default(),
    )
    .unwrap();

    if parsed_toml.pipeline.is_some() {
        // Handle standard pipeline
        let mutation_chain: MutationChain = parsed_toml.pipeline.clone().unwrap().into();
        let mutated_log = mutation_chain.apply(&log);

        write_xes(
            &mutated_log,
            parsed_toml
                .clone()
                .output
                .unwrap()
                .to_string_lossy()
                .to_string(),
            true,
        )?;
    } else if parsed_toml.parametrized_pipeline.is_some() {
        // Handle parametrized pipeline
        let mutation_config_vecs = parsed_toml
            .parametrized_pipeline
            .clone()
            .unwrap()
            .to_mutation_config_vec_vec();

        for vec in mutation_config_vecs {
            // Path creation
            let mut path = get_parametrized_pipeline_output_root(&parsed_toml)?;
            path.push_str(mutation_config_vec_to_path(&vec).as_str());
            path.push_str("/log.xes");
            if parsed_toml.compress_output {
                path.push_str(".gz");
            }

            // Apply mutations
            let mutation_chain: MutationChain = PipelineConfig::new(vec).into();
            let mutated_log = mutation_chain.apply(&log);

            // Write event log file
            write_xes(&mutated_log, path.clone(), parsed_toml.compress_output)?;
            println!("Wrote event log: {}", path);
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn create_road_traffic_time_logs() {
    const PROBABILITIES: [f32; 14] = [
        0.0, 0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0,
    ];
    // const STANDARD_DEVIATIONS: [f64; 11] = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
    const STANDARD_DEVIATIONS: [f64; 14] = [
        0.0, 0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0,
    ];

    let log = import_xes_file("./road_traffic.xes.gz", XESImportOptions::default()).unwrap();
    let case_ids: Vec<String> = log
        .traces
        .iter()
        .map(|trace| get_traceid(trace).unwrap())
        .collect();

    let rng = &mut rand::thread_rng();
    for prob in PROBABILITIES {
        for std in STANDARD_DEVIATIONS {
            println!(
                "Creating Event Log for probability {} and std shift {}",
                prob, std
            );
            let output_path = format!("./road_traffic_newlogs/probability_{}/std_{}", prob, std);

            if Path::new(format!("{output_path}/log_1.xes.gz").as_str()).exists() {
                println!("\talready exists... skip");
                continue;
            }

            let mut mutated_log = ServiceTimeStdShifter::new(std)
                .with_probability(prob)
                .for_activity("Send Fine")
                .apply(&log);

            // Get random sample of half the caseids
            let sample: HashSet<String> = case_ids
                .choose_multiple(rng, case_ids.len() / 2)
                .cloned()
                .collect();

            mutated_log
                .traces
                .retain(|trace| sample.contains(&get_traceid(trace).unwrap()));

            let mut non_mutated_log = log.clone();
            non_mutated_log
                .traces
                .retain(|trace| !sample.contains(&get_traceid(trace).unwrap()));

            // Save the files
            write_xes(
                &non_mutated_log,
                format!("{}/log_1.xes.gz", output_path),
                true,
            )
            .unwrap();
            write_xes(&mutated_log, format!("{}/log_2.xes.gz", output_path), true).unwrap();
        }
    }
}

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

    let input = args.clone().input.unwrap();
    if args.output.is_none() {
        args.output = Some(get_output_path(&input));
    }

    if args.no_overwrite && args.output.clone().unwrap().exists() {
        Err(CliError::new(
            ErrorKind::Io,
            "The output path already exists and `--no-overwrite` specified. Aborting.",
        ))?
    }

    if input.exists() && input.is_file() {
        let log = import_xes_file(input.to_str().unwrap(), XESImportOptions::default()).unwrap();

        let mutation_chain = if let Some(preset) = args.preset {
            println!("Using preset {:?}", preset);
            preset.into_mutation_chain(&log, args.clone())
        } else {
            let mut chain =
                MutationChain::new().with_mutation(LogBootstrapper::new(log.traces.len()));
            if args.mutate {
                println!("Applying mutations...");
                chain = chain.with_mutation(
                    ServiceTimeMultiplier::new(2.0)
                        .for_activity("W_Completeren aanvraag")
                        .with_probability(1.0),
                )
                // .with_mutation(EventSwapper::new("A_SUBMITTED", "A_PARTLYSUBMITTED"))
                // .with_mutation(
                //     // Only 270 instances in the original log --> ~ <600 in bootstrapped
                //     ActivityRemover::new("W_Beoordelen fraude".to_owned()).with_probability(1.0),
                // );
            }
            chain
        };

        let l = mutation_chain.apply(&log);
        let should_compress = args
            .output
            .clone()
            .unwrap()
            .extension()
            .map_or(false, |ext| ext == "gz");

        write_xes(
            &l,
            args.output.unwrap().to_string_lossy().to_string(),
            should_compress,
        )?
    } else {
        return Err(CliError::new(
            ErrorKind::Io,
            "The input file does not exist, or is not a file.",
        ));
    }
    Ok(())
}

fn write_xes(log: &EventLog, path: String, compress: bool) -> Result<(), CliError> {
    let p: &Path = Path::new(&path);
    let dir_creation_res = p.parent().map(create_dir_all);
    if dir_creation_res.is_none() || dir_creation_res.unwrap().is_err() {
        return Err(CliError::new(
            ErrorKind::Io,
            format!(
                "Something went wrong creating the directories on the path {}",
                path
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

#[allow(dead_code)]
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
