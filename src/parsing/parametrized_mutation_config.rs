use crate::{
    mutators::filters::case_duration_filter::ComparisonSense,
    parsing::mutation_value::MutationValue, parsing::traits::FlattenMutationValue,
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
        seed: Option<MutationValue<u64>>,
    },
    ActivityRenamer {
        activity: MutationValue<String>,
        new_label: MutationValue<String>,
        #[serde(default = "default_probability_mutation_value")]
        probability: MutationValue<f32>,
        seed: Option<MutationValue<u64>>,
    },
    AttributeRetainer {
        attributes: MutationValue<Vec<String>>,
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
