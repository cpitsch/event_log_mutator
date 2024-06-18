use serde::Deserialize;

use itertools::{iproduct, Itertools};

use super::{MutationChainConfig, MutationConfig, PipelineConfig};

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MutationValue<T> {
    Value(T),
    Vec(Vec<T>),
}

impl<T> MutationValue<T> {
    pub fn normalized(self) -> Self {
        match self {
            Self::Value(x) => Self::Vec(vec![x]),
            Self::Vec(v) => Self::Vec(v),
        }
    }

    // TODO: Typestate pattern? Only expose this if it has been normalized
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
    MutationValue::Value(1.0)
}

fn default_standard_deviations_mutation_value() -> MutationValue<f64> {
    MutationValue::Value(1.0)
}

fn default_service_time_factor_mutation_value() -> MutationValue<f32> {
    MutationValue::Value(1.0)
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

impl ParametrizedMutationConfig {
    pub fn normalized(self) -> Self {
        // Make all mutation-values vecs
        match self {
            Self::ServiceTimeStdShifter {
                activity,
                probability,
                standard_deviations,
            } => Self::ServiceTimeStdShifter {
                activity: activity.normalized(),
                probability: probability.normalized(),
                standard_deviations: standard_deviations.normalized(),
            },
            Self::VariantSupportFilter {
                num_supporting_cases,
            } => Self::VariantSupportFilter {
                num_supporting_cases: num_supporting_cases.normalized(),
            },
            Self::ActivityRemover {
                activity,
                probability,
            } => Self::ActivityRemover {
                activity: activity.normalized(),
                probability: probability.normalized(),
            },
            Self::ActivityRenamer {
                activity,
                new_label,
                probability,
            } => Self::ActivityRenamer {
                activity: activity.normalized(),
                new_label: new_label.normalized(),
                probability: probability.normalized(),
            },
            Self::ConstantActivity {
                activity,
                probability,
            } => Self::ConstantActivity {
                activity: activity.normalized(),
                probability: probability.normalized(),
            },
            Self::EventSwapper {
                activity_1,
                activity_2,
                probability,
            } => Self::EventSwapper {
                activity_1: activity_1.normalized(),
                activity_2: activity_2.normalized(),
                probability: probability.normalized(),
            },
            Self::LogBootstrapper { size } => Self::LogBootstrapper {
                size: size.normalized(),
            },
            Self::PartialOrderCreator => Self::PartialOrderCreator,
            Self::AttributeRemover { key } => Self::AttributeRemover {
                key: key.normalized(),
            },
            Self::ServiceTimeMultiplier {
                activity,
                probability,
                factor,
            } => Self::ServiceTimeMultiplier {
                activity: activity.normalized(),
                probability: probability.normalized(),
                factor: factor.normalized(),
            },
        }
    }
    pub fn to_mutation_config_vec(self) -> Vec<MutationConfig> {
        match self.normalized() {
            Self::ServiceTimeStdShifter {
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
            Self::VariantSupportFilter {
                num_supporting_cases,
            } => num_supporting_cases
                .get_as_vec()
                .iter()
                .map(|threshold| MutationConfig::VariantSupportFilter {
                    num_supporting_cases: *threshold,
                })
                .collect(),

            Self::ServiceTimeMultiplier {
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
            Self::AttributeRemover { key } => key
                .get_as_vec()
                .into_iter()
                .map(|k| MutationConfig::AttributeRemover { key: k })
                .collect(),
            Self::PartialOrderCreator => vec![MutationConfig::PartialOrderCreator],
            Self::EventSwapper {
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
            Self::LogBootstrapper { size } => size
                .get_as_vec()
                .iter()
                .map(|s| MutationConfig::LogBootstrapper { size: *s })
                .collect(),
            Self::ConstantActivity {
                activity,
                probability,
            } => iproduct!(activity.get_as_vec(), probability.get_as_vec())
                .map(|(act, prob)| MutationConfig::ConstantActivity {
                    activity: act,
                    probability: prob,
                })
                .collect(),
            Self::ActivityRenamer {
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
            Self::ActivityRemover {
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

impl ParametrizedPipelineConfig {
    pub fn to_pipeline_config_vec(self) -> Vec<PipelineConfig> {
        self.to_mutation_config_vec_vec()
            .into_iter()
            .map(|v| PipelineConfig { mutations: v })
            .collect()
    }

    pub fn to_mutation_config_vec_vec(self) -> Vec<Vec<MutationConfig>> {
        self.mutations
            .iter()
            .cloned()
            .map(ParametrizedMutationConfig::to_mutation_config_vec)
            .multi_cartesian_product()
            .collect_vec()
    }
}

pub fn get_parametrized_pipeline_output_root(config: &MutationChainConfig) -> String {
    let mut base_path = if let Some(out) = &config.output {
        if out.is_file() {
            panic!("Parametrized pipeline cannot take file as output path")
        }
        out.as_os_str().to_string_lossy().to_string()
    } else {
        "./".to_string()
    };
    if !base_path.ends_with('/') {
        base_path.push('/');
    }
    base_path
}
