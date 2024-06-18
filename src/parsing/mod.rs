use std::{fs::read_to_string, path::PathBuf};

pub mod parametrized_pipeline;

use itertools::Itertools;
use serde::Deserialize;
use toml::from_str;

use crate::{
    mutation::{LogMutator, MutationChain},
    mutators::{
        filters::VariantSupportFilter, ActivityRemover, ActivityRenamer, AttributeRemover,
        ConstantActivityMutator, EventSwapper, LogBootstrapper, PartialOrderCreator,
        ServiceTimeMultiplier, ServiceTimeStdShifter,
    },
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
    /// A definition for a standard mutation pipeline
    pub pipeline: Option<PipelineConfig>,
    /// A definition for a parametrized mutation pipeline
    pub parametrized_pipeline: Option<ParametrizedPipelineConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub enum PipelineEnum {
    StandardPipeline(PipelineConfig),
    ParametrizedPipeline(ParametrizedPipelineConfig),
}

#[derive(Deserialize, Debug, Clone)]
pub struct PipelineConfig {
    pub mutations: Vec<MutationConfig>,
}

// #[derive(Deserialize, Debug)]
// pub struct MutationConfig {
//     #[serde(rename = "type")]
//     mutation_type: String,
//     parameters: ParametersConfig,
// }

fn default_probability() -> f32 {
    1.0
}

fn default_standard_deviations() -> f64 {
    1.0
}

fn default_service_time_factor() -> f32 {
    1.0
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

impl From<MutationConfig> for Box<dyn LogMutator> {
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
            MutationConfig::LogBootstrapper { size } => Box::new(LogBootstrapper::new(size)),
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

impl MutationConfig {
    /// Used to derive unique save-paths for parametrized pipelines.
    pub fn as_dir_name(&self) -> String {
        match self {
            MutationConfig::ServiceTimeStdShifter {
                activity,
                probability,
                standard_deviations,
            } => format!(
                "ServiceTimeStdShifter_{}_p{}_std{}",
                activity.clone().unwrap_or("AllActivities".to_string()),
                probability,
                standard_deviations
            ),
            MutationConfig::VariantSupportFilter {
                num_supporting_cases,
            } => format!("VariantSupportFilter_thresh{}", num_supporting_cases),

            MutationConfig::ActivityRemover {
                activity,
                probability,
            } => format!("ActivityRemover_{}_p{}", activity, probability),
            MutationConfig::ActivityRenamer {
                activity,
                new_label,
                probability,
            } => format!(
                "ActivityRenamer_from_{}_to_{}_p{}",
                activity, new_label, probability
            ),
            MutationConfig::ConstantActivity {
                activity,
                probability,
            } => format!("ConstantActivity_{}_p{}", activity, probability),
            MutationConfig::EventSwapper {
                activity_1,
                activity_2,
                probability,
            } => format!(
                "EventSwapper_{}_swap_{}_p{}",
                activity_1, activity_2, probability
            ),
            MutationConfig::LogBootstrapper { size } => format!("LogBootstrapper_{}", size),
            MutationConfig::PartialOrderCreator => "PartialOrderCreator".to_string(),
            MutationConfig::AttributeRemover { key } => format!("AttributeRemover_{}", key),
            MutationConfig::ServiceTimeMultiplier {
                activity,
                probability,
                factor,
            } => format!(
                "ServiceTimeMultiplier_{}_p{}_x{}",
                activity.clone().unwrap_or("All Activities".to_string()),
                probability,
                factor
            ),
        }
    }
}

impl PipelineConfig {
    pub fn new(mutations: Vec<MutationConfig>) -> Self {
        Self { mutations }
    }
}

impl From<PipelineConfig> for MutationChain {
    fn from(value: PipelineConfig) -> Self {
        let mut chain = MutationChain::new();
        for mutation_config in value.mutations {
            chain.mutations.push(mutation_config.into())
        }
        chain
    }
}

// #[derive(Deserialize, Debug)]
// pub struct ParametersConfig {
//     activity: Option<String>,
//     probability: Option<f64>,
//     standard_deviations: Option<f64>,
//     num_supporting_cases: i64,
// }

pub fn parse_toml(path: &PathBuf) -> MutationChainConfig {
    let contents = read_to_string(path).unwrap();
    let res: MutationChainConfig = from_str(&contents).expect("Invalid TOML format");

    if res.pipeline.is_some() && res.parametrized_pipeline.is_some() {
        // TODO: Make this an error, and return a result
        panic!("Pipeline and Parametrized Pipeline defined. Only one allowed")
    }

    res
}

pub fn mutation_config_vec_to_path(mutation_configs: &[MutationConfig]) -> String {
    mutation_configs
        .iter()
        .map(|conf| conf.as_dir_name())
        .join("/")
}

// impl From<PipelineConfig> for MutationChain {
//     fn from(value: PipelineConfig) -> Self {
//         let mut mutation_chain = Self::new();
//         for mutation_config in value.mutations {
//             let mutation: Box<dyn LogMutator> = match mutation_config.mutation_type {
//
//             }
//         }
//     }
// }
