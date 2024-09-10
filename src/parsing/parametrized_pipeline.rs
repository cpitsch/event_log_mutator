use serde::Deserialize;

use itertools::Itertools;

use crate::{
    mutation::{LogMutatorWithAsDirName, MutationChain},
    mutators::{
        filters::{CaseDurationFilter, EndpointFilter, VariantSupportFilter},
        ActivityRemover, ActivityRenamer, AttributeRemover, ConstantActivityMutator, EventSwapper,
        LogBootstrapper, LogSplitter, PartialOrderCreator, ServiceTimeMultiplier,
        ServiceTimeStdShifter,
    },
    parsing::flatten_mutation_value_trait::FlattenMutationValue,
    CliError,
};

use super::{dir_name_trait::DirName, MutationChainConfig};

#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
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
            Self::Vec(_) => panic!("Called inner_value on non-flat MutationValue"),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
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

#[derive(Deserialize, Debug, Clone, FlattenMutationValue)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(tag = "type")]
pub enum ParametrizedMutationConfig {
    ServiceTimeStdShifter {
        activity: Option<MutationValue<String>>,
        #[serde(default = "default_standard_deviations_mutation_value")]
        standard_deviations: MutationValue<f64>,
        #[serde(default = "default_probability_mutation_value")]
        probability: MutationValue<f32>,
        seed: Option<MutationValue<u64>>,
    },
    VariantSupportFilter {
        num_supporting_cases: MutationValue<usize>,
    },
    EndpointFilter {
        start_activities: Option<MutationValue<Vec<String>>>,
        end_activities: Option<MutationValue<Vec<String>>>,
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
        seed: Option<MutationValue<u64>>,
    },
    ActivityRenamer {
        activity: MutationValue<String>,
        new_label: MutationValue<String>,
        #[serde(default = "default_probability_mutation_value")]
        probability: MutationValue<f32>,
        seed: Option<MutationValue<u64>>,
    },
    ConstantActivity {
        activity: MutationValue<String>,
        #[serde(default = "default_probability_mutation_value")]
        probability: MutationValue<f32>,
        seed: Option<MutationValue<u64>>,
    },
    EventSwapper {
        activity_1: MutationValue<String>,
        activity_2: MutationValue<String>,
        #[serde(default = "default_probability_mutation_value")]
        probability: MutationValue<f32>,
        seed: Option<MutationValue<u64>>,
    },
    LogSplitter {
        frac: MutationValue<f64>,
        save_path: Option<MutationValue<String>>,
        save_compressed: Option<MutationValue<bool>>,
        seed: Option<MutationValue<u64>>,
    },
    LogBootstrapper {
        size: MutationValue<usize>,
        #[serde(default = "default_log_bootstrapper_replacement_value")]
        replacement: MutationValue<bool>,
        seed: Option<MutationValue<u64>>,
    },
    PartialOrderCreator,
    AttributeRemover {
        key: MutationValue<String>,
    },
    ServiceTimeMultiplier {
        activity: Option<MutationValue<String>>,
        #[serde(default = "default_probability_mutation_value")]
        probability: MutationValue<f32>,
        #[serde(default = "default_service_time_factor_mutation_value")]
        factor: MutationValue<f32>,
        seed: Option<MutationValue<u64>>,
    },
}

pub fn flat_mutation_config_to_log_mutator(
    flat_config: ParametrizedMutationConfig,
    root_seed: Option<u64>,
    dir_so_far: String,
) -> Box<dyn LogMutatorWithAsDirName> {
    match flat_config {
        ParametrizedMutationConfig::ServiceTimeStdShifter {
            activity,
            probability,
            standard_deviations,
            seed,
        } => {
            let mut mutator = ServiceTimeStdShifter::new(standard_deviations.inner_value())
                .with_probability(probability.inner_value());
            if let Some(act) = activity {
                mutator = mutator.for_activity(act.inner_value());
            }
            if let Some(s) = seed.map(|s| s.inner_value()).or(root_seed) {
                mutator = mutator.with_seed(s);
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
            start_activities.map(MutationValue::inner_value),
            end_activities.map(MutationValue::inner_value),
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
            seed,
        } => {
            let mut mutator = ActivityRemover::new(activity.inner_value())
                .with_probability(probability.inner_value());
            if let Some(s) = seed.map(|s| s.inner_value()).or(root_seed) {
                mutator = mutator.with_seed(s);
            }
            Box::new(mutator)
        }
        ParametrizedMutationConfig::ActivityRenamer {
            activity,
            new_label,
            probability,
            seed,
        } => {
            let mut mutator = ActivityRenamer::new(activity.inner_value(), new_label.inner_value())
                .with_probability(probability.inner_value());
            if let Some(s) = seed.map(|s| s.inner_value()).or(root_seed) {
                mutator = mutator.with_seed(s);
            }
            Box::new(mutator)
        }
        ParametrizedMutationConfig::ConstantActivity {
            activity,
            probability,
            seed,
        } => {
            let mut mutator = ConstantActivityMutator::new(activity.inner_value())
                .with_probability(probability.inner_value());
            if let Some(s) = seed.map(|s| s.inner_value()).or(root_seed) {
                mutator = mutator.with_seed(s);
            }
            Box::new(mutator)
        }
        ParametrizedMutationConfig::EventSwapper {
            activity_1,
            activity_2,
            probability,
            seed,
        } => {
            let mut mutator = EventSwapper::new(activity_1.inner_value(), activity_2.inner_value())
                .with_probability(probability.inner_value());
            if let Some(s) = seed.map(|s| s.inner_value()).or(root_seed) {
                mutator = mutator.with_seed(s);
            }
            Box::new(mutator)
        }
        ParametrizedMutationConfig::LogSplitter {
            frac,
            save_path,
            save_compressed,
            seed,
        } => {
            let mut mutator = LogSplitter::new(frac.inner_value());
            if let Some(s) = seed.map(MutationValue::inner_value).or(root_seed) {
                mutator = mutator.with_seed(s);
            }
            if let Some(p) = save_path {
                mutator = mutator.save_discarded(p.inner_value());
            } else {
                let mut save_path = dir_so_far.clone();
                save_path.push_str(&format!("{}/log.xes", mutator.to_dir_name()).to_string());
                if save_compressed
                    .clone()
                    .unwrap_or(MutationValue::Value(false))
                    .inner_value()
                {
                    save_path.push_str(".gz");
                }

                mutator = mutator.save_discarded(save_path);
            }
            if let Some(c) = save_compressed {
                mutator = mutator.save_compressed(c.inner_value());
            }
            Box::new(mutator)
        }
        ParametrizedMutationConfig::LogBootstrapper {
            size,
            replacement,
            seed,
        } => {
            let mut mutator = LogBootstrapper::new(size.inner_value())
                .with_replacement(replacement.inner_value());
            if let Some(s) = seed.map(|s| s.inner_value()).or(root_seed) {
                mutator = mutator.with_seed(s);
            }
            Box::new(mutator)
        }
        ParametrizedMutationConfig::PartialOrderCreator => Box::new(PartialOrderCreator::new()),
        ParametrizedMutationConfig::AttributeRemover { key } => {
            Box::new(AttributeRemover::new(key.inner_value()))
        }
        ParametrizedMutationConfig::ServiceTimeMultiplier {
            activity,
            probability,
            factor,
            seed,
        } => {
            let mut mutator = ServiceTimeMultiplier::new(factor.inner_value())
                .with_probability(probability.inner_value());
            if let Some(act) = activity {
                mutator = mutator.for_activity(act.inner_value());
            }
            if let Some(s) = seed.map(|s| s.inner_value()).or(root_seed) {
                mutator = mutator.with_seed(s);
            }
            Box::new(mutator)
        }
    }
}

// pub fn parametrized_mutation_config_vec_to_mutation_chain_vec(
//     configs: Vec<ParametrizedMutationConfig>,
//     root_seed: Option<u64>,
// ) -> Vec<MutationChain> {
//     configs
//         .into_iter()
//         .map(ParametrizedMutationConfig::flatten)
//         .multi_cartesian_product()
//         .map(|flat_configs| MutationChain {
//             mutations: flat_configs
//                 .into_iter()
//                 .map(|flat_config| flat_mutation_config_to_log_mutator(flat_config, root_seed))
//                 .collect(),
//         })
//         .collect()
// }

pub fn parametrized_mutation_config_vec_to_mutation_chain_vec(
    configs: Vec<ParametrizedMutationConfig>,
    root_seed: Option<u64>,
    output_root: String,
) -> Vec<MutationChain> {
    configs
        .into_iter()
        .map(ParametrizedMutationConfig::flatten)
        .multi_cartesian_product()
        .map(|flat_configs| {
            let mut mutations: Vec<Box<dyn LogMutatorWithAsDirName>> =
                Vec::with_capacity(flat_configs.len());
            let mut dir_so_far = output_root.clone();
            flat_configs.into_iter().for_each(|flat_config| {
                let mutator = flat_mutation_config_to_log_mutator(
                    flat_config,
                    root_seed,
                    dir_so_far.to_string(),
                );
                dir_so_far.push_str(format!("/{}", mutator.to_dir_name()).as_str());
                mutations.push(mutator);
            });
            MutationChain { mutations }
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
