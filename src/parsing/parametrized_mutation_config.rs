use crate::parsing::custom_serde::deserialize_u64_vec_or_range_option;
use std::path::PathBuf;

use crate::{
    mutators::filters::{
        attribute_value_filter::{AttributeFilterMethod, AttributeFilterTarget},
        ComparisonSense,
    },
    parsing::{mutation_value::MutationValue, traits::FlattenMutationValue},
};
use serde::Deserialize;

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

#[derive(Deserialize, Debug, Clone, FlattenMutationValue)]
#[serde(tag = "type")]
pub enum ParametrizedMutationConfig {
    ServiceTimeStdShifter {
        activity: Option<MutationValue<String>>,
        #[serde(default = "default_standard_deviations_mutation_value")]
        standard_deviations: MutationValue<f64>,
        #[serde(default = "default_probability_mutation_value")]
        probability: MutationValue<f32>,
        #[serde(default, deserialize_with = "deserialize_u64_vec_or_range_option")]
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
        #[serde(default)]
        sense: MutationValue<ComparisonSense>,
        #[serde(default)]
        years: MutationValue<f32>,
        #[serde(default)]
        days: MutationValue<f32>,
        #[serde(default)]
        hours: MutationValue<f32>,
        #[serde(default)]
        minutes: MutationValue<f32>,
        #[serde(default)]
        seconds: MutationValue<f32>,
    },
    ActivityRemover {
        activity: MutationValue<String>,
        #[serde(default = "default_probability_mutation_value")]
        probability: MutationValue<f32>,
        #[serde(default, deserialize_with = "deserialize_u64_vec_or_range_option")]
        seed: Option<MutationValue<u64>>,
    },
    ActivityRenamer {
        activity: MutationValue<String>,
        new_label: MutationValue<String>,
        #[serde(default = "default_probability_mutation_value")]
        probability: MutationValue<f32>,
        #[serde(default, deserialize_with = "deserialize_u64_vec_or_range_option")]
        seed: Option<MutationValue<u64>>,
    },
    AttributeRetainer {
        attributes: MutationValue<Vec<String>>,
    },
    ConstantActivity {
        activity: MutationValue<String>,
        #[serde(default = "default_probability_mutation_value")]
        probability: MutationValue<f32>,
        #[serde(default, deserialize_with = "deserialize_u64_vec_or_range_option")]
        seed: Option<MutationValue<u64>>,
    },
    EventSwapper {
        activity_1: MutationValue<String>,
        activity_2: MutationValue<String>,
        #[serde(default = "default_probability_mutation_value")]
        probability: MutationValue<f32>,
        #[serde(default, deserialize_with = "deserialize_u64_vec_or_range_option")]
        seed: Option<MutationValue<u64>>,
    },
    LogSplitter {
        frac: MutationValue<f64>,
        save_path: Option<MutationValue<PathBuf>>,
        save_compressed: Option<MutationValue<bool>>,
        #[serde(default, deserialize_with = "deserialize_u64_vec_or_range_option")]
        seed: Option<MutationValue<u64>>,
    },
    // WARN: LogBootstrapper is the deprecated name for LogSampler and may be
    // deleted at any time
    #[serde(alias = "LogBootstrapper")]
    LogSampler {
        size: MutationValue<usize>,
        #[serde(default = "default_log_bootstrapper_replacement_value")]
        replacement: MutationValue<bool>,
        #[serde(default, deserialize_with = "deserialize_u64_vec_or_range_option")]
        seed: Option<MutationValue<u64>>,
    },
    // WARN: PartialOrderCreator is the deprecated name for LogSampler and may be
    // deleted at any time
    #[serde(alias = "PartialOrderCreator")]
    SojournStartAdder {
        key: Option<MutationValue<String>>,
    },
    AttributeRemover {
        key: MutationValue<String>,
    },
    ServiceTimeMultiplier {
        activity: Option<MutationValue<String>>,
        #[serde(default = "default_probability_mutation_value")]
        probability: MutationValue<f32>,
        #[serde(default = "default_service_time_factor_mutation_value")]
        factor: MutationValue<f32>,
        #[serde(default, deserialize_with = "deserialize_u64_vec_or_range_option")]
        seed: Option<MutationValue<u64>>,
    },
    FollowerFilter {
        trigger_activities: MutationValue<Vec<String>>,
        reaction_activities: MutationValue<Vec<String>>,
        range: Option<MutationValue<usize>>,
    },
    AttributeFilter {
        target: MutationValue<AttributeFilterTarget>,
        key: MutationValue<String>,
        #[serde(flatten)]
        filter_method: AttributeFilterMethod,
    },
    TraceLengthFilter {
        length: MutationValue<usize>,
        sense: Option<MutationValue<ComparisonSense>>,
    },
}
