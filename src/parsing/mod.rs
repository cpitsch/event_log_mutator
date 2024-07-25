use std::{fs::read_to_string, path::PathBuf};

pub mod dir_name_trait;
pub mod parametrized_pipeline;

use itertools::Itertools;
use serde::Deserialize;
use toml::from_str;

use crate::{
    mutation::{LogMutatorWithAsDirName, MutationChain},
    mutators::{
        filters::{CaseDurationFilter, EndpointFilter, VariantSupportFilter},
        ActivityRemover, ActivityRenamer, AttributeRemover, ConstantActivityMutator, EventSwapper,
        LogBootstrapper, PartialOrderCreator, ServiceTimeMultiplier, ServiceTimeStdShifter,
    },
    CliError,
};

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

fn default_probability() -> f32 {
    1.0
}

fn default_standard_deviations() -> f64 {
    1.0
}

fn default_service_time_factor() -> f32 {
    1.0
}

fn default_log_bootstrapper_replacement() -> bool {
    true
}

fn zero() -> f32 {
    0.0
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum MutationConfig {
    ServiceTimeStdShifter {
        activity: Option<String>,
        #[serde(default = "default_probability")]
        probability: f32,
        #[serde(default = "default_standard_deviations")]
        standard_deviations: f64,
    },
    VariantSupportFilter {
        num_supporting_cases: usize,
    },
    EndpointFilter {
        start_activities: Option<Vec<String>>,
        end_activities: Option<Vec<String>>,
    },
    CaseDurationFilter {
        #[serde(default = "zero")]
        years: f32,
        #[serde(default = "zero")]
        days: f32,
        #[serde(default = "zero")]
        hours: f32,
        #[serde(default = "zero")]
        minutes: f32,
        #[serde(default = "zero")]
        seconds: f32,
    },
    ActivityRemover {
        activity: String,
        #[serde(default = "default_probability")]
        probability: f32,
    },
    ActivityRenamer {
        activity: String,
        new_label: String,
        #[serde(default = "default_probability")]
        probability: f32,
    },
    ConstantActivity {
        activity: String,
        #[serde(default = "default_probability")]
        probability: f32,
    },
    EventSwapper {
        activity_1: String,
        activity_2: String,
        #[serde(default = "default_probability")]
        probability: f32,
    },
    LogBootstrapper {
        size: usize,
        #[serde(default = "default_log_bootstrapper_replacement")]
        replacement: bool,
    },
    PartialOrderCreator,
    AttributeRemover {
        key: String,
    },
    ServiceTimeMultiplier {
        activity: Option<String>,
        #[serde(default = "default_probability")]
        probability: f32,
        #[serde(default = "default_service_time_factor")]
        factor: f32,
    },
}

impl From<MutationConfig> for Box<dyn LogMutatorWithAsDirName> {
    fn from(val: MutationConfig) -> Self {
        match val {
            MutationConfig::ServiceTimeStdShifter {
                activity,
                probability,
                standard_deviations,
            } => {
                let mut mutator =
                    ServiceTimeStdShifter::new(standard_deviations).with_probability(probability);
                if let Some(act) = activity {
                    mutator = mutator.for_activity(act);
                }
                Box::new(mutator)
            }
            MutationConfig::VariantSupportFilter {
                num_supporting_cases,
            } => Box::new(VariantSupportFilter::new(num_supporting_cases)),
            MutationConfig::EndpointFilter {
                start_activities,
                end_activities,
            } => Box::new(EndpointFilter::new(start_activities, end_activities)),
            MutationConfig::CaseDurationFilter {
                years,
                days,
                hours,
                minutes,
                seconds,
            } => Box::new(CaseDurationFilter::new(
                Some(years),
                Some(days),
                Some(hours),
                Some(minutes),
                Some(seconds),
            )),
            MutationConfig::ActivityRemover {
                activity,
                probability,
            } => Box::new(ActivityRemover::new(activity).with_probability(probability)),
            MutationConfig::ActivityRenamer {
                activity,
                new_label,
                probability,
            } => Box::new(ActivityRenamer::new(activity, new_label).with_probability(probability)),
            MutationConfig::ConstantActivity {
                activity,
                probability,
            } => Box::new(ConstantActivityMutator::new(activity).with_probability(probability)),
            MutationConfig::EventSwapper {
                activity_1,
                activity_2,
                probability,
            } => Box::new(EventSwapper::new(activity_1, activity_2).with_probability(probability)),
            MutationConfig::LogBootstrapper { size, replacement } => {
                Box::new(LogBootstrapper::new(size).with_replacement(replacement))
            }
            MutationConfig::PartialOrderCreator => Box::new(PartialOrderCreator::new()),
            MutationConfig::AttributeRemover { key } => Box::new(AttributeRemover::new(key)),
            MutationConfig::ServiceTimeMultiplier {
                activity,
                probability,
                factor,
            } => {
                let mut mutator = ServiceTimeMultiplier::new(factor).with_probability(probability);
                if let Some(act) = activity {
                    mutator = mutator.for_activity(act);
                }
                Box::new(mutator)
            }
        }
    }
}

impl From<Vec<MutationConfig>> for MutationChain {
    fn from(val: Vec<MutationConfig>) -> Self {
        MutationChain {
            mutations: val.into_iter().map_into().collect(),
        }
    }
}

pub fn parse_toml(path: &PathBuf) -> Result<MutationChainConfig, CliError> {
    let contents = read_to_string(path).unwrap();
    let res: MutationChainConfig = from_str(&contents).expect("Invalid TOML format");
    Ok(res)
}
