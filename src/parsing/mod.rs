use std::{fs::read_to_string, path::PathBuf};

pub mod dir_name_trait;
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
}

pub fn parse_toml(path: &PathBuf) -> Result<MutationChainConfig, CliError> {
    let contents = read_to_string(path).unwrap();
    let res: MutationChainConfig = from_str(&contents).expect("Invalid TOML format");
    Ok(res)
}
