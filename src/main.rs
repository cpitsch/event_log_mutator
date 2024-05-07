use std::{
    fs::File,
    path::{Path, PathBuf},
};

use clap::Parser;
use mutators::{ActivityRemover, EventSwapper, ServiceTimeMultiplier};
use process_mining::{
    event_log::export_xes::export_xes_event_log_to_file, import_xes_file, XESImportOptions,
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
}

fn main() {
    let mut args = Args::parse();

    if args.output.is_none() {
        args.output = Some(get_output_path(&args.input));
    }

    if args.input.exists() && args.input.is_file() {
        let log =
            import_xes_file(args.input.to_str().unwrap(), XESImportOptions::default()).unwrap();

        let mutation_chain = MutationChain::new()
            .with_mutation(ActivityRemover::new("receive goods".to_owned(), 0.2))
            .with_mutation(ActivityRemover::new("pay invoice".to_owned(), 0.25))
            // .with_mutation(ActivityRenamer::new(
            //     "manager reject purchase".to_owned(),
            //     "manager disapproval".to_owned(),
            //     1.0,
            // ))
            .with_mutation(EventSwapper::new(
                "inspect goods".to_owned(),
                "receive invoice".to_owned(),
                1.0,
            ))
            .with_mutation(ServiceTimeMultiplier::new(2.0));
        let l = mutation_chain.apply(&log);
        let file = File::create(args.output.unwrap()).unwrap();
        export_xes_event_log_to_file(&l, file, false).unwrap();
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
