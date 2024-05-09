use std::{
    fs::File,
    path::{Path, PathBuf},
};

use clap::{Parser, ValueEnum};
use mutators::{
    partial_order_creator::PartialOrderCreator, ActivityRemover, LogBootstrapper,
    ServiceTimeMultiplier,
};
use process_mining::{
    event_log::export_xes::export_xes_event_log_to_file, import_xes_file, EventLog,
    XESImportOptions,
};

use crate::mutation::{LogMutator, MutationChain};

pub mod constants;
pub mod mutation;
pub mod mutators;
pub mod utils;

#[derive(Parser, Debug)]
struct Args {
    /// The input XES file
    #[clap(short, long)]
    input: PathBuf,

    /// The path to write the mutated log to. Defaults to /path/to/input_mutated.xes
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// If present, apply mutations to the event log. Otherwise, only apply bootstrapping.
    #[clap(long)]
    mutate: bool,

    #[clap(long, value_enum)]
    preset: Option<Preset>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Preset {
    Bpi12OnlyServiceTime,
    Bpi12,
    /// Turn an atomic event log into a partially ordered event log by using the
    /// time since the previous event as the service time.
    PartialOrder,
}

impl Preset {
    pub fn into_mutation_chain(self, log: &EventLog, mutate: bool) -> MutationChain {
        match self {
            Self::Bpi12 => {
                let mut mutation_chain =
                    MutationChain::new().with_mutation(LogBootstrapper::new(log.traces.len()));

                if mutate {
                    mutation_chain = mutation_chain
                        .with_mutation(
                            ServiceTimeMultiplier::new(2.0)
                                .for_activity("W_Completeren aanvraag")
                                .with_probability(1.0),
                        )
                        .with_mutation(
                            // Only 270 instances in the original log
                            ActivityRemover::new("W_Beoordelen fraude").with_probability(1.0),
                        );
                }

                mutation_chain
            }
            Self::Bpi12OnlyServiceTime => MutationChain::new()
                .with_mutation(LogBootstrapper::new(log.traces.len()))
                .with_mutation(
                    ServiceTimeMultiplier::new(2.0)
                        .for_activity("W_Completeren aanvraag")
                        .with_probability(1.0),
                ),
            Self::PartialOrder => MutationChain::new(), //.with_mutation(PartialOrderCreator::new()),
        }
    }
}

fn main() {
    let mut args = Args::parse();
    if args.output.is_none() {
        args.output = Some(get_output_path(&args.input));
    }

    if args.input.exists() && args.input.is_file() {
        let log =
            import_xes_file(args.input.to_str().unwrap(), XESImportOptions::default()).unwrap();

        let mutation_chain = if let Some(preset) = args.preset {
            println!("Using preset {:?}", preset);
            preset.into_mutation_chain(&log, args.mutate)
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
        let file = File::create(args.output.unwrap()).unwrap();
        export_xes_event_log_to_file(&l, file, should_compress).unwrap();
    }
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

// let mutation_chain = MutationChain::new()
//     .with_mutation(ActivityRemover::new("receive goods").with_probability(0.2))
//     .with_mutation(ActivityRemover::new("pay invoice").with_probability(0.25))
//     // .with_mutation(ActivityRenamer::new(
//     //     "manager reject purchase",
//     //     "manager disapproval",
//     //     1.0,
//     // ))
//     .with_mutation(
//         EventSwapper::new("inspect goods", "receive invoice")
//             .with_probability(1.0),
//     )
//     .with_mutation(
//         ServiceTimeMultiplier::new(2.0).for_activity("inspect goods"),
//     );
