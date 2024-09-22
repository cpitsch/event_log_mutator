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
    parsing::{
        mutation_value::MutationValue,
        parametrized_mutation_config::ParametrizedMutationConfig,
        traits::{DirName, FlattenMutationValue},
    },
};

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Flat;
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct NotFlat;

#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ParametrizedPipelineConfig<State = NotFlat> {
    pub mutations: Vec<ParametrizedMutationConfig>,
    #[serde(skip)]
    _state: std::marker::PhantomData<State>,
}

impl ParametrizedPipelineConfig {
    pub fn new(mutations: Vec<ParametrizedMutationConfig>) -> ParametrizedPipelineConfig<NotFlat> {
        Self {
            mutations,
            _state: std::marker::PhantomData::<NotFlat>,
        }
    }
}

impl ParametrizedPipelineConfig<NotFlat> {
    pub fn flatten(self) -> Vec<ParametrizedPipelineConfig<Flat>> {
        self.mutations
            .into_iter()
            .map(ParametrizedMutationConfig::flatten)
            .multi_cartesian_product()
            .map(|chain| ParametrizedPipelineConfig {
                mutations: chain,
                _state: std::marker::PhantomData::<Flat>,
            })
            .collect()
    }

    pub fn to_mutation_chains(
        self,
        root_seed: Option<u64>,
        output_root: String,
    ) -> Vec<MutationChain> {
        self.flatten()
            .into_iter()
            .map(|flat_pipeline_config| {
                flat_pipeline_config.into_mutation_chain(root_seed, output_root.clone())
            })
            .collect()
    }
}

impl ParametrizedPipelineConfig<Flat> {
    pub fn into_mutation_chain(
        self,
        root_seed: Option<u64>,
        mut output_root: String,
    ) -> MutationChain {
        let mut mutations: Vec<Box<dyn LogMutatorWithAsDirName>> =
            Vec::with_capacity(self.mutations.len());
        self.mutations.into_iter().for_each(|flat_config| {
            let mutator = Self::flat_mutation_config_to_log_mutator(
                flat_config,
                root_seed,
                output_root.to_string(),
            );
            output_root.push_str(format!("/{}", mutator.to_dir_name()).as_str());
            mutations.push(mutator);
        });
        MutationChain { mutations }
    }

    fn flat_mutation_config_to_log_mutator(
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
                sense,
                years,
                days,
                hours,
                minutes,
                seconds,
            } => Box::new(
                CaseDurationFilter::new(
                    Some(years.inner_value()),
                    Some(days.inner_value()),
                    Some(hours.inner_value()),
                    Some(minutes.inner_value()),
                    Some(seconds.inner_value()),
                )
                .with_sense(sense.inner_value()),
            ),
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
                let mut mutator =
                    ActivityRenamer::new(activity.inner_value(), new_label.inner_value())
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
                let mut mutator =
                    EventSwapper::new(activity_1.inner_value(), activity_2.inner_value())
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
                    if !save_path.is_empty() {
                        save_path.push('/')
                    }
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
}
