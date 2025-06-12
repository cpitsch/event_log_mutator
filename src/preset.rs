use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use clap::Subcommand;
use process_mining::{import_xes_file, EventLog, XESImportOptions};

use crate::{
    cli::{Args, CliError, CliResult},
    mutation::{LogMutator, MutationChain, MutationError},
    mutators::{
        aux_mutators::LogSaver, filters::VariantSupportFilter, LogBootstrapper, PartialOrderCreator,
    },
    utils::io::{ensure_correct_file_extension, IoError},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum Preset {
    /// Bootstrap a "new" event log of the same size by sampling cases with replacement
    Bootstrap {
        /// Number of cases to sample. Defaults to the log length.
        size: Option<usize>,
        /// Sample without replacement?
        #[clap(long)]
        no_replacement: bool,
    },
    /// Turn an atomic event log into a partially ordered event log by using the
    /// time since the previous event as the service time
    PartialOrder,
    /// Retain only the cases whose variant is supported by at least `n` cases total
    FilterVariantSupport {
        /// Minimum number of supporting cases to keep a variant.
        support: usize,
    },
}

impl Preset {
    pub fn into_mutation_chain(self, log: &EventLog, seed: Option<u64>) -> MutationChain {
        match self {
            Self::Bootstrap {
                size,
                no_replacement,
            } => {
                let mut bootstrapper = LogBootstrapper::new(size.unwrap_or(log.traces.len()))
                    .with_replacement(!no_replacement);
                if let Some(seed) = seed {
                    bootstrapper = bootstrapper.with_seed(seed);
                }
                MutationChain::new().with_mutation(bootstrapper)
            }
            Self::PartialOrder => MutationChain::new().with_mutation(PartialOrderCreator::new()),
            Self::FilterVariantSupport { support } => {
                MutationChain::new().with_mutation(VariantSupportFilter::new(support))
            }
        }
    }

    pub fn execute(self, args: Args, no_overwrite: bool) -> CliResult<()> {
        let input = args
            .input
            .as_ref()
            .ok_or(CliError::MissingRequiredArgument(
                "If no pipeline file (--pipeline) is provided, an input file must be specified (--input)",
            ))?;

        let mut output = args
            .output
            .clone()
            .map_or_else(|| Self::get_output_path(input), Ok)?;
        let should_compress = output.extension().is_some_and(|ext| ext == "gz");
        output = ensure_correct_file_extension(output, should_compress);

        if no_overwrite && output.exists() {
            Err(IoError::FileExists(output.clone()))?
        }

        if input.is_file() {
            let mut log = import_xes_file(input, XESImportOptions::default())?;
            self.into_mutation_chain(&log, args.seed)
                .with_mutation(LogSaver::new(output, should_compress))
                .apply_mut(&mut log)?;

            Ok(())
        } else {
            Err(IoError::FileNotFound(input.clone()))?
        }
    }

    fn get_output_path(input_path: &Path) -> CliResult<PathBuf> {
        let mut out = PathBuf::new();
        if let Some(parent) = input_path.parent() {
            out.push(parent);
        }

        let name_string = input_path.file_name().ok_or_else(|| {
            MutationError::InvalidValue("The input path should end in a file name".into())
        })?;
        // Prepend 'mutated_' and call it a day
        let mut filename = OsString::from("mutated_");
        filename.push(name_string);

        out.push(filename);
        Ok(out)
    }
}
