use std::{fs::read_to_string, path::PathBuf};

pub mod dir_name_trait;
pub mod flatten_mutation_value_trait;
pub mod parametrized_pipeline;

use serde::Deserialize;
use toml::from_str;

use crate::CliError;

use self::parametrized_pipeline::ParametrizedPipelineConfig;

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
    /// Seed to use for mutations involving randomness.
    /// Overwritten by seeds set on a mutation-level.
    pub seed: Option<u64>,
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

    use tests::parametrized_pipeline::ParametrizedMutationConfig;

    use super::parametrized_pipeline::MutationValue;
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
        assert_eq!(
            res.output,
            Some(PathBuf::from_str("output.xes.gz").unwrap())
        );

        assert!(!res.compress_output); // Not provided, default to false
        assert_eq!(res.seed, None);

        assert_eq!(
            res.pipeline,
            ParametrizedPipelineConfig {
                mutations: vec![
                    ParametrizedMutationConfig::ServiceTimeStdShifter {
                        activity: MutationValue::Value(Some("a".to_string())),
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
                ]
            }
        )
    }
}
