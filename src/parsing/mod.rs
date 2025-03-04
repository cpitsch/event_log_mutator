use std::{
    ffi::OsString,
    fs::read_to_string,
    path::{Path, PathBuf},
};

pub mod custom_serde;
pub mod mutation_value;
pub mod parametrized_mutation_config;
pub mod parametrized_pipeline;
pub mod traits;

use log::info;
use process_mining::{import_xes_file, XESImportOptions};
use serde::Deserialize;
use thiserror::Error;
use toml::from_str;

use crate::{
    cli::CliError,
    mutation::{LogMutator, MutationError},
    mutators::aux_mutators::LogValidator,
    parsing::parametrized_pipeline::LogAction,
    utils::io::{ensure_correct_file_extension, write_xes},
};

use self::parametrized_pipeline::{
    flattened_pipeline_configs_to_mutation_chains, ParametrizedPipelineConfig,
};

#[derive(Deserialize, Debug, Clone)]
pub struct MutationChainConfig {
    /// The path to the input event log
    pub input: PathBuf,
    /// The path to write the event log to.
    /// For a parametrized pipeline, this is the root directory of the save paths
    pub output: Option<PathBuf>,
    /// Save the output log(s) gzipped. Defaults to false
    #[serde(default)] // Default to default bool (false)
    pub compress_output: bool,
    /// A definition for a mutation pipeline
    pub pipeline: ParametrizedPipelineConfig,
}

#[derive(Error, Debug)]
pub enum ParsingError {
    #[error("Invalid value in TOML file: {0}")]
    InvalidValue(&'static str),
    #[error(transparent)]
    TomlDeserializationError(#[from] toml::de::Error),
    #[error("The TOML file {0:?} does not exist")]
    FileNotFoundError(PathBuf),
}

impl MutationChainConfig {
    pub fn default_output_path(&self, is_parametrized: bool) -> PathBuf {
        if is_parametrized {
            PathBuf::from(".")
        } else {
            self.input.with_file_name(format!(
                "mutated_{}",
                self.input.file_name().unwrap().to_string_lossy()
            ))
        }
    }

    /// Parse a pipeline configuration from a TOML file
    pub fn parse_file(path: impl AsRef<Path>) -> Result<Self, ParsingError> {
        let path = path.as_ref();
        if !path.is_file() {
            return Err(ParsingError::FileNotFoundError(path.to_path_buf()));
        }
        let contents = read_to_string(path).unwrap();
        Self::parse_toml_str(&contents)
    }

    pub fn parse_toml_str(content: &str) -> Result<Self, ParsingError> {
        Ok(from_str::<Self>(content)?)
    }

    pub fn validate(&self) -> Result<(), CliError> {
        self.run_with_log_action(LogAction::Validate(self.compress_output))
    }
    pub fn execute(&self) -> Result<(), CliError> {
        self.run_with_log_action(LogAction::Save(self.compress_output))
    }

    fn run_with_log_action(&self, log_action: LogAction) -> Result<(), CliError> {
        let mut pipelines = self.pipeline.clone().flatten();

        // If effectively only one mutation config, you should be able to provide a specific
        // output file instead of an output root path
        if pipelines.len() == 1 {
            let mut output_path = self
                .output
                .clone()
                .unwrap_or_else(|| self.default_output_path(false));

            // Read the event log. Since there is only one mutation chain, we can
            // mutate the event log directly
            let mut log = import_xes_file(&self.input, XESImportOptions::default())?;
            info!("Read event log {}", self.input.to_string_lossy());

            if output_path.extension().is_none() {
                // No extension -> interpret as only directories in the path; Add file name
                // Technically don't need this error, as if the event log imported successfully,
                // then it is a file..
                let name_string = self.input.file_name().ok_or_else(|| {
                    MutationError::InvalidValue("The input path should end in a file name".into())
                })?;
                let mut filename = OsString::from("mutated_");
                filename.push(name_string);
                output_path.push(filename)
            }
            output_path = ensure_correct_file_extension(output_path, self.compress_output);

            let mut mutation_chain = pipelines.pop().unwrap().into_mutation_chain(
                output_path.clone(),
                // Don't save the output implicitly through the MutationChain; Do it here explicitly
                &LogAction::None,
            );
            mutation_chain.apply_mut(&mut log)?;

            match log_action {
                LogAction::Validate(_compressed) => {
                    LogValidator::new(output_path).apply_mut(&mut log)?;
                }
                LogAction::Save(compressed) => write_xes(&log, output_path, compressed)?,
                LogAction::None => {}
            };
        } else {
            let output_path = self
                .output
                .clone()
                .unwrap_or_else(|| self.default_output_path(true));

            // Read the event log
            let log = import_xes_file(&self.input, XESImportOptions::default())?;
            info!("Read event log {}", self.input.to_string_lossy());

            for mut mutation_chain in
                flattened_pipeline_configs_to_mutation_chains(pipelines, &output_path, log_action)
                    .into_iter()
            {
                mutation_chain.apply(&log)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use tests::parametrized_mutation_config::ParametrizedMutationConfig;

    use crate::test_fixtures::mininmal_toml_example;

    use super::mutation_value::MutationValue;
    use super::*;

    const TOML_CONTENT: &str = "
input = \"input_log.xes.gz\"
output = \"output.xes.gz\"

[pipeline]
[[pipeline.mutations]]
type = \"ServiceTimeStdShifter\"
activity = \"a\"
probability = 0.5
standard_deviations = 1.0
seed=42

[[pipeline.mutations]]
type = \"ActivityRenamer\"
activity = \"Send Fine\"
new_label = \"New Activity\"
probability = 0.5
    ";

    #[test]
    fn config_params_parsed_correctly() {
        let res = MutationChainConfig::parse_toml_str(TOML_CONTENT).unwrap();

        assert_eq!(res.input, PathBuf::from("input_log.xes.gz"));
        assert_eq!(res.output, Some(PathBuf::from("output.xes.gz")));

        assert!(!res.compress_output); // Not provided, default to false

        assert_eq!(
            res.pipeline,
            ParametrizedPipelineConfig::new(vec![
                ParametrizedMutationConfig::ServiceTimeStdShifter {
                    activity: Some(MutationValue::Value("a".to_string())),
                    standard_deviations: MutationValue::Value(1.0),
                    probability: MutationValue::Value(0.5),
                    seed: Some(MutationValue::Value(42))
                },
                ParametrizedMutationConfig::ActivityRenamer {
                    activity: MutationValue::Value("Send Fine".to_string()),
                    new_label: MutationValue::Value("New Activity".to_string()),
                    probability: MutationValue::Value(0.5),
                    seed: None
                }
            ])
        )
    }

    #[rstest]
    fn minimal_example_parses(mininmal_toml_example: &str) {
        let _ = MutationChainConfig::parse_toml_str(mininmal_toml_example);
    }
}
