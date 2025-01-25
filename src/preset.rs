use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use clap::ValueEnum;
use process_mining::{import_xes_file, EventLog, XESImportOptions};

use crate::{
    cli::{Args, CliError, CliResult},
    mutation::{LogMutator, MutationChain, MutationError},
    mutators::{filters::VariantSupportFilter, LogBootstrapper, PartialOrderCreator},
    utils::io::{ensure_correct_file_extension, write_xes, IoError},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Preset {
    /// Bootstrap a "new" event log of the same size by sampling cases with replacement
    Bootstrap,
    /// Turn an atomic event log into a partially ordered event log by using the
    /// time since the previous event as the service time
    PartialOrder,
    /// Retain only the cases whose variant is supported by at least `n` cases total
    FilterVariantSupport,
}

impl Preset {
    pub fn into_mutation_chain(self, log: &EventLog, args: Args) -> MutationChain {
        match self {
            Self::Bootstrap => {
                MutationChain::new().with_mutation(LogBootstrapper::new(log.traces.len()))
            }
            Self::PartialOrder => MutationChain::new().with_mutation(PartialOrderCreator::new()),
            Self::FilterVariantSupport => {
                MutationChain::new()
                    .with_mutation(VariantSupportFilter::new(args.support.expect(
                        "Variant Support Filter requires the `--support` flag to be set.",
                    )))
            }
        }
    }

    pub fn execute(args: Args) -> CliResult<()> {
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

        if args.no_overwrite && output.exists() {
            Err(IoError::FileExists(output.clone()))?
        }

        if input.is_file() {
            let mut log = import_xes_file(&input.to_string_lossy(), XESImportOptions::default())?;
            if let Some(preset) = args.preset {
                preset
                    .into_mutation_chain(&log, args.clone())
                    .apply_mut(&mut log)?;

                let should_compress = output.extension().is_some_and(|ext| ext == "gz");
                output = ensure_correct_file_extension(output, should_compress);
                Ok(write_xes(&log, output, should_compress)?)
            } else {
                Err(CliError::MissingRequiredArgument(
                    "Either a pipeline file (--pipeline) or a preset (--preset) must be provided!",
                ))
            }
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
