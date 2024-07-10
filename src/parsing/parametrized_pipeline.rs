use serde::Deserialize;

use itertools::{iproduct, Itertools};

use crate::{mutation::MutationChain, CliError};

use super::{
    default_log_bootstrapper_replacement, default_probability, default_service_time_factor,
    default_standard_deviations, MutationChainConfig, MutationConfig,
};

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MutationValue<T> {
    Value(T),
    Vec(Vec<T>),
}

impl<T> MutationValue<T> {
    pub fn get_as_vec(self) -> Vec<T> {
        match self {
            Self::Vec(v) => v,
            Self::Value(v) => vec![v],
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ParametrizedPipelineConfig {
    pub mutations: Vec<ParametrizedMutationConfig>,
}

fn default_probability_mutation_value() -> MutationValue<f32> {
    MutationValue::Value(default_probability())
}

fn default_standard_deviations_mutation_value() -> MutationValue<f64> {
    MutationValue::Value(default_standard_deviations())
}

fn default_service_time_factor_mutation_value() -> MutationValue<f32> {
    MutationValue::Value(default_service_time_factor())
}

fn default_log_bootstrapper_replacement_value() -> MutationValue<bool> {
    MutationValue::Value(default_log_bootstrapper_replacement())
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ParametrizedMutationConfig {
    ServiceTimeStdShifter {
        activity: MutationValue<Option<String>>,
        #[serde(default = "default_probability_mutation_value")]
        probability: MutationValue<f32>,
        #[serde(default = "default_standard_deviations_mutation_value")]
        standard_deviations: MutationValue<f64>,
    },
    VariantSupportFilter {
        num_supporting_cases: MutationValue<usize>,
    },
    ActivityRemover {
        activity: MutationValue<String>,
        #[serde(default = "default_probability_mutation_value")]
        probability: MutationValue<f32>,
    },
    ActivityRenamer {
        activity: MutationValue<String>,
        new_label: MutationValue<String>,
        #[serde(default = "default_probability_mutation_value")]
        probability: MutationValue<f32>,
    },
    ConstantActivity {
        activity: MutationValue<String>,
        #[serde(default = "default_probability_mutation_value")]
        probability: MutationValue<f32>,
    },
    EventSwapper {
        activity_1: MutationValue<String>,
        activity_2: MutationValue<String>,
        #[serde(default = "default_probability_mutation_value")]
        probability: MutationValue<f32>,
    },
    LogBootstrapper {
        size: MutationValue<usize>,
        #[serde(default = "default_log_bootstrapper_replacement_value")]
        replacement: MutationValue<bool>,
    },
    PartialOrderCreator,
    AttributeRemover {
        key: MutationValue<String>,
    },
    ServiceTimeMultiplier {
        activity: MutationValue<Option<String>>,
        #[serde(default = "default_probability_mutation_value")]
        probability: MutationValue<f32>,
        #[serde(default = "default_service_time_factor_mutation_value")]
        factor: MutationValue<f32>,
    },
}

impl From<ParametrizedPipelineConfig> for Vec<MutationChain> {
    fn from(value: ParametrizedPipelineConfig) -> Self {
        std::convert::Into::<Vec<Vec<MutationConfig>>>::into(value)
            .into_iter()
            .map_into()
            .collect()
    }
}

impl From<ParametrizedMutationConfig> for Vec<MutationConfig> {
    fn from(value: ParametrizedMutationConfig) -> Self {
        match value {
            ParametrizedMutationConfig::ServiceTimeStdShifter {
                activity,
                probability,
                standard_deviations,
            } => iproduct!(
                activity.get_as_vec(),
                probability.get_as_vec(),
                standard_deviations.get_as_vec()
            )
            .map(|(act, prob, std)| MutationConfig::ServiceTimeStdShifter {
                activity: act,
                probability: prob,
                standard_deviations: std,
            })
            .collect(),
            ParametrizedMutationConfig::VariantSupportFilter {
                num_supporting_cases,
            } => num_supporting_cases
                .get_as_vec()
                .iter()
                .map(|threshold| MutationConfig::VariantSupportFilter {
                    num_supporting_cases: *threshold,
                })
                .collect(),

            ParametrizedMutationConfig::ServiceTimeMultiplier {
                activity,
                probability,
                factor,
            } => iproduct!(
                activity.get_as_vec(),
                probability.get_as_vec(),
                factor.get_as_vec()
            )
            .map(
                |(act, prob, factor)| MutationConfig::ServiceTimeMultiplier {
                    activity: act,
                    probability: prob,
                    factor,
                },
            )
            .collect(),
            ParametrizedMutationConfig::AttributeRemover { key } => key
                .get_as_vec()
                .into_iter()
                .map(|k| MutationConfig::AttributeRemover { key: k })
                .collect(),
            ParametrizedMutationConfig::PartialOrderCreator => {
                vec![MutationConfig::PartialOrderCreator]
            }
            ParametrizedMutationConfig::EventSwapper {
                activity_1,
                activity_2,
                probability,
            } => iproduct!(
                activity_1.get_as_vec(),
                activity_2.get_as_vec(),
                probability.get_as_vec()
            )
            .map(|(act_1, act_2, prob)| MutationConfig::EventSwapper {
                activity_1: act_1,
                activity_2: act_2,
                probability: prob,
            })
            .collect(),
            ParametrizedMutationConfig::LogBootstrapper { size, replacement } => {
                iproduct!(size.get_as_vec(), replacement.get_as_vec())
                    .map(|(s, replace)| MutationConfig::LogBootstrapper {
                        size: s,
                        replacement: replace,
                    })
                    .collect()
            }
            ParametrizedMutationConfig::ConstantActivity {
                activity,
                probability,
            } => iproduct!(activity.get_as_vec(), probability.get_as_vec())
                .map(|(act, prob)| MutationConfig::ConstantActivity {
                    activity: act,
                    probability: prob,
                })
                .collect(),
            ParametrizedMutationConfig::ActivityRenamer {
                activity,
                new_label,
                probability,
            } => iproduct!(
                activity.get_as_vec(),
                new_label.get_as_vec(),
                probability.get_as_vec()
            )
            .map(|(act, label, prob)| MutationConfig::ActivityRenamer {
                activity: act,
                new_label: label,
                probability: prob,
            })
            .collect(),
            ParametrizedMutationConfig::ActivityRemover {
                activity,
                probability,
            } => iproduct!(activity.get_as_vec(), probability.get_as_vec())
                .map(|(act, prob)| MutationConfig::ActivityRemover {
                    activity: act,
                    probability: prob,
                })
                .collect(),
        }
    }
}

impl From<ParametrizedPipelineConfig> for Vec<Vec<MutationConfig>> {
    fn from(value: ParametrizedPipelineConfig) -> Self {
        value
            .mutations
            .iter()
            .cloned()
            .map_into::<Vec<MutationConfig>>()
            .multi_cartesian_product()
            .collect_vec()
    }
}

pub fn get_parametrized_pipeline_output_root(
    config: &MutationChainConfig,
) -> Result<String, CliError> {
    let mut base_path = if let Some(out) = &config.output {
        if out.is_file() {
            return Err(CliError::new(
                clap::error::ErrorKind::ValueValidation,
                "Parametrized pipeline cannot take file as output path",
            ));
        }
        out.as_os_str().to_string_lossy().to_string()
    } else {
        "./".to_string()
    };
    if !base_path.ends_with('/') {
        base_path.push('/');
    }
    Ok(base_path)
}
