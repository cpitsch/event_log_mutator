use serde::Deserialize;

use itertools::{iproduct, Itertools};

use crate::{
    mutation::{LogMutatorWithAsDirName, MutationChain},
    mutators::{
        filters::{CaseDurationFilter, EndpointFilter, VariantSupportFilter},
        ActivityRemover, ActivityRenamer, AttributeRemover, ConstantActivityMutator, EventSwapper,
        LogBootstrapper, PartialOrderCreator, ServiceTimeMultiplier, ServiceTimeStdShifter,
    },
    CliError,
};

use super::MutationChainConfig;

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

    pub fn inner_value(self) -> T {
        match self {
            Self::Value(v) => v,
            Self::Vec(_) => panic!("Called get_value on non-flat MutationValue"),
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

fn default_log_bootstrapper_replacement_value() -> MutationValue<bool> {
    MutationValue::Value(true)
}

fn zero_f32_mutation_value() -> MutationValue<f32> {
    MutationValue::Value(0.0)
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
    EndpointFilter {
        start_activities: MutationValue<Option<Vec<String>>>,
        end_activities: MutationValue<Option<Vec<String>>>,
    },
    CaseDurationFilter {
        #[serde(default = "zero_f32_mutation_value")]
        years: MutationValue<f32>,
        #[serde(default = "zero_f32_mutation_value")]
        days: MutationValue<f32>,
        #[serde(default = "zero_f32_mutation_value")]
        hours: MutationValue<f32>,
        #[serde(default = "zero_f32_mutation_value")]
        minutes: MutationValue<f32>,
        #[serde(default = "zero_f32_mutation_value")]
        seconds: MutationValue<f32>,
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

impl ParametrizedMutationConfig {
    fn flatten(self) -> Vec<Self> {
        match self {
            ParametrizedMutationConfig::ServiceTimeStdShifter {
                activity,
                probability,
                standard_deviations,
            } => iproduct!(
                activity.get_as_vec(),
                probability.get_as_vec(),
                standard_deviations.get_as_vec()
            )
            .map(
                |(act, prob, std)| ParametrizedMutationConfig::ServiceTimeStdShifter {
                    activity: MutationValue::Value(act),
                    probability: MutationValue::Value(prob),
                    standard_deviations: MutationValue::Value(std),
                },
            )
            .collect(),
            ParametrizedMutationConfig::VariantSupportFilter {
                num_supporting_cases,
            } => num_supporting_cases
                .get_as_vec()
                .iter()
                .map(
                    |threshold| ParametrizedMutationConfig::VariantSupportFilter {
                        num_supporting_cases: MutationValue::Value(*threshold),
                    },
                )
                .collect(),
            ParametrizedMutationConfig::EndpointFilter {
                start_activities,
                end_activities,
            } => iproduct!(start_activities.get_as_vec(), end_activities.get_as_vec())
                .map(
                    |(start_acts, end_acts)| ParametrizedMutationConfig::EndpointFilter {
                        start_activities: MutationValue::Value(start_acts),
                        end_activities: MutationValue::Value(end_acts),
                    },
                )
                .collect(),
            ParametrizedMutationConfig::CaseDurationFilter {
                years,
                days,
                hours,
                minutes,
                seconds,
            } => iproduct!(
                years.get_as_vec(),
                days.get_as_vec(),
                hours.get_as_vec(),
                minutes.get_as_vec(),
                seconds.get_as_vec()
            )
            .map(|(years, days, hours, minutes, seconds)| {
                ParametrizedMutationConfig::CaseDurationFilter {
                    years: MutationValue::Value(years),
                    days: MutationValue::Value(days),
                    hours: MutationValue::Value(hours),
                    minutes: MutationValue::Value(minutes),
                    seconds: MutationValue::Value(seconds),
                }
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
                |(act, prob, factor)| ParametrizedMutationConfig::ServiceTimeMultiplier {
                    activity: MutationValue::Value(act),
                    probability: MutationValue::Value(prob),
                    factor: MutationValue::Value(factor),
                },
            )
            .collect(),
            ParametrizedMutationConfig::AttributeRemover { key } => key
                .get_as_vec()
                .into_iter()
                .map(|k| ParametrizedMutationConfig::AttributeRemover {
                    key: MutationValue::Value(k),
                })
                .collect(),
            ParametrizedMutationConfig::PartialOrderCreator => {
                vec![ParametrizedMutationConfig::PartialOrderCreator]
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
            .map(
                |(act_1, act_2, prob)| ParametrizedMutationConfig::EventSwapper {
                    activity_1: MutationValue::Value(act_1),
                    activity_2: MutationValue::Value(act_2),
                    probability: MutationValue::Value(prob),
                },
            )
            .collect(),
            ParametrizedMutationConfig::LogBootstrapper { size, replacement } => {
                iproduct!(size.get_as_vec(), replacement.get_as_vec())
                    .map(|(s, replace)| ParametrizedMutationConfig::LogBootstrapper {
                        size: MutationValue::Value(s),
                        replacement: MutationValue::Value(replace),
                    })
                    .collect()
            }
            ParametrizedMutationConfig::ConstantActivity {
                activity,
                probability,
            } => iproduct!(activity.get_as_vec(), probability.get_as_vec())
                .map(|(act, prob)| ParametrizedMutationConfig::ConstantActivity {
                    activity: MutationValue::Value(act),
                    probability: MutationValue::Value(prob),
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
            .map(
                |(act, label, prob)| ParametrizedMutationConfig::ActivityRenamer {
                    activity: MutationValue::Value(act),
                    new_label: MutationValue::Value(label),
                    probability: MutationValue::Value(prob),
                },
            )
            .collect(),
            ParametrizedMutationConfig::ActivityRemover {
                activity,
                probability,
            } => iproduct!(activity.get_as_vec(), probability.get_as_vec())
                .map(|(act, prob)| ParametrizedMutationConfig::ActivityRemover {
                    activity: MutationValue::Value(act),
                    probability: MutationValue::Value(prob),
                })
                .collect(),
        }
    }
}

pub fn parametrized_mutation_config_vec_to_mutation_chain_vec(
    configs: Vec<ParametrizedMutationConfig>,
) -> Vec<MutationChain> {
    configs
        .into_iter()
        .map(ParametrizedMutationConfig::flatten)
        .multi_cartesian_product()
        .map(|flat_configs| MutationChain {
            mutations: flat_configs
                .into_iter()
                .map(|flat_config| -> Box<dyn LogMutatorWithAsDirName> {
                    match flat_config {
                        ParametrizedMutationConfig::ServiceTimeStdShifter {
                            activity,
                            probability,
                            standard_deviations,
                        } => {
                            let mut mutator =
                                ServiceTimeStdShifter::new(standard_deviations.inner_value())
                                    .with_probability(probability.inner_value());
                            if let Some(act) = activity.inner_value() {
                                mutator = mutator.for_activity(act);
                            }
                            Box::new(mutator)
                        }
                        ParametrizedMutationConfig::VariantSupportFilter {
                            num_supporting_cases,
                        } => Box::new(VariantSupportFilter::new(
                            num_supporting_cases.inner_value(),
                        )),
                        ParametrizedMutationConfig::EndpointFilter {
                            start_activities,
                            end_activities,
                        } => Box::new(EndpointFilter::new(
                            start_activities.inner_value(),
                            end_activities.inner_value(),
                        )),
                        ParametrizedMutationConfig::CaseDurationFilter {
                            years,
                            days,
                            hours,
                            minutes,
                            seconds,
                        } => Box::new(CaseDurationFilter::new(
                            Some(years.inner_value()),
                            Some(days.inner_value()),
                            Some(hours.inner_value()),
                            Some(minutes.inner_value()),
                            Some(seconds.inner_value()),
                        )),
                        ParametrizedMutationConfig::ActivityRemover {
                            activity,
                            probability,
                        } => Box::new(
                            ActivityRemover::new(activity.inner_value())
                                .with_probability(probability.inner_value()),
                        ),
                        ParametrizedMutationConfig::ActivityRenamer {
                            activity,
                            new_label,
                            probability,
                        } => Box::new(
                            ActivityRenamer::new(activity.inner_value(), new_label.inner_value())
                                .with_probability(probability.inner_value()),
                        ),
                        ParametrizedMutationConfig::ConstantActivity {
                            activity,
                            probability,
                        } => Box::new(
                            ConstantActivityMutator::new(activity.inner_value())
                                .with_probability(probability.inner_value()),
                        ),
                        ParametrizedMutationConfig::EventSwapper {
                            activity_1,
                            activity_2,
                            probability,
                        } => Box::new(
                            EventSwapper::new(activity_1.inner_value(), activity_2.inner_value())
                                .with_probability(probability.inner_value()),
                        ),
                        ParametrizedMutationConfig::LogBootstrapper { size, replacement } => {
                            Box::new(
                                LogBootstrapper::new(size.inner_value())
                                    .with_replacement(replacement.inner_value()),
                            )
                        }
                        ParametrizedMutationConfig::PartialOrderCreator => {
                            Box::new(PartialOrderCreator::new())
                        }
                        ParametrizedMutationConfig::AttributeRemover { key } => {
                            Box::new(AttributeRemover::new(key.inner_value()))
                        }
                        ParametrizedMutationConfig::ServiceTimeMultiplier {
                            activity,
                            probability,
                            factor,
                        } => {
                            let mut mutator = ServiceTimeMultiplier::new(factor.inner_value())
                                .with_probability(probability.inner_value());
                            if let Some(act) = activity.inner_value() {
                                mutator = mutator.for_activity(act);
                            }
                            Box::new(mutator)
                        }
                    }
                })
                .collect(),
        })
        .collect()
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
