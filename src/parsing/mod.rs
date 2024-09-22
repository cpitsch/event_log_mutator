use std::{fs::read_to_string, path::PathBuf, str::FromStr};

pub mod mutation_value;
pub mod parametrized_mutation_config;
pub mod parametrized_pipeline;
pub mod traits;

use process_mining::{import_xes_file, EventLog, XESImportOptions};
use serde::Deserialize;
use toml::from_str;

use crate::{mutation::LogMutator, write_xes, CliError};

use self::parametrized_pipeline::ParametrizedPipelineConfig;

fn default_output_pathbuf() -> PathBuf {
    PathBuf::from_str("./").unwrap()
}

#[derive(Deserialize, Debug, Clone)]
pub struct MutationChainConfig {
    /// The path to the input event log
    pub input: PathBuf,
    /// The path to write the event log to.
    /// For a parametrized pipeline, this is the root directory of the save paths
    #[serde(default = "default_output_pathbuf")]
    pub output: PathBuf,
    /// Save the output log(s) gzipped. Defaults to false
    #[serde(default)] // Default to default bool (false)
    pub compress_output: bool,
    /// A definition for a mutation pipeline
    pub pipeline: ParametrizedPipelineConfig,
    /// Seed to use for mutations involving randomness.
    /// Overwritten by seeds set on a mutation-level.
    pub seed: Option<u64>,
}

impl MutationChainConfig {
    pub fn execute(&self) -> Result<(), CliError> {
        let mut mutation_chains = self
            .pipeline
            .clone()
            .to_mutation_chains(self.seed, &self.output);

        // If effectively only one mutation config, you should be able to provide a specific
        // output file instead of an output root path
        if mutation_chains.len() == 1 {
            // Read the event log
            let mut log =
                import_xes_file(&self.input.to_string_lossy(), XESImportOptions::default())
                    .unwrap();

            let mut output_path = self.output.clone();
            if !output_path.ends_with(".xes") && !output_path.ends_with(".xes.gz") {
                output_path.push("mutated_log.xes");
            }
            let mut mutation_chain = mutation_chains.pop().unwrap();
            mutation_chain.apply_mut(&mut log);

            write_xes(&log, output_path, self.compress_output)?;
        } else {
            for mut mutation_chain in mutation_chains {
                if self.output.is_file() {
                    return Err(CliError::new(
                        clap::error::ErrorKind::InvalidValue,
                        "For a parametrized pipeline, the output path may not be a file.",
                    ));
                }

                // Read the event log
                let log =
                    import_xes_file(&self.input.to_string_lossy(), XESImportOptions::default())
                        .unwrap();

                // Path creation
                let mut path = self.output.clone();
                mutation_chain
                    .mutations
                    .iter()
                    .for_each(|mutation| path.push(mutation.to_dir_name()));
                path.push("log_2.xes");
                if self.compress_output {
                    path.set_extension("xes.gz");
                }

                // Apply mutations
                let mutated_log = mutation_chain.apply(&log);

                // Write event log file
                write_xes(&mutated_log, path.clone(), self.compress_output)?;
                println!("Wrote event log: {}", path.to_string_lossy());
            }
        }

        Ok(())
    }
}

pub fn parse_toml(path: &PathBuf) -> Result<MutationChainConfig, CliError> {
    let contents = read_to_string(path).unwrap();
    let res = parse_toml_string(&contents);
    Ok(res)
}

pub fn parse_toml_string(content: &str) -> MutationChainConfig {
    from_str(content).expect("Invalid TOML format")
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

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
        let res = parse_toml_string(TOML_CONTENT);

        assert_eq!(res.input, PathBuf::from_str("input_log.xes.gz").unwrap());
        assert_eq!(res.output, PathBuf::from_str("output.xes.gz").unwrap());

        assert!(!res.compress_output); // Not provided, default to false
        assert_eq!(res.seed, None);

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
        let _ = parse_toml_string(mininmal_toml_example);
    }
}
