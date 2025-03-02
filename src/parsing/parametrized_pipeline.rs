use std::path::{Path, PathBuf};

use chrono::TimeDelta;
use serde::Deserialize;

use itertools::Itertools;

use crate::{
    mutation::{LogMutatorWithAsDirName, MutationChain},
    mutators::{
        aux_mutators::{LogSaver, LogValidator},
        filters::{CaseDurationFilter, EndpointFilter, VariantSupportFilter},
        ActivityRemover, ActivityRenamer, AttributeRemover, AttributeRetainer,
        ConstantActivityMutator, EventSwapper, LogBootstrapper, LogSplitter, PartialOrderCreator,
        ServiceTimeMultiplier, ServiceTimeStdShifter,
    },
    parsing::{
        custom_serde::deserialize_u64_vec_or_range_option,
        mutation_value::MutationValue,
        parametrized_mutation_config::ParametrizedMutationConfig,
        traits::{DirName, FlattenMutationValue},
    },
    utils::io::{build_file_path, ensure_correct_file_extension},
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
    /// Seed to use for mutations involving randomness.
    /// Overwritten by seeds set on a mutation-level.
    #[serde(default, deserialize_with = "deserialize_u64_vec_or_range_option")]
    pub seed: Option<MutationValue<u64>>,
    #[serde(skip)]
    _state: std::marker::PhantomData<State>,
}

// What to do with the mutated event log?
#[derive(Debug)]
pub enum LogAction {
    Save(bool),     // Save --> compressed?
    Validate(bool), // Is the target event log compressed?
    None,
}

impl ParametrizedPipelineConfig {
    pub fn new(mutations: Vec<ParametrizedMutationConfig>) -> ParametrizedPipelineConfig<NotFlat> {
        Self {
            mutations,
            seed: None,
            _state: std::marker::PhantomData::<NotFlat>,
        }
    }
}

impl ParametrizedPipelineConfig<NotFlat> {
    pub fn flatten(self) -> Vec<ParametrizedPipelineConfig<Flat>> {
        let flattened = self
            .mutations
            .into_iter()
            .map(ParametrizedMutationConfig::flatten)
            .multi_cartesian_product()
            .map(|config| ParametrizedPipelineConfig {
                mutations: config,
                seed: None,
                _state: std::marker::PhantomData::<Flat>,
            });

        if let Some(seeds) = self.seed.map(MutationValue::get_as_vec) {
            flattened
                .cartesian_product(seeds)
                .map(|(config, seed)| config.with_seed(seed))
                .collect()
        } else {
            flattened.collect()
        }
    }

    pub fn to_mutation_chains(
        self,
        output_root: &Path,
        log_action: LogAction,
    ) -> Vec<MutationChain> {
        flattened_pipeline_configs_to_mutation_chains(self.flatten(), output_root, log_action)
    }
}

/// Convert flattened pipeline objects (ParametrizedPipelineConfig<Flat>) to a vector
/// of MutationChains.
///
/// * `pipelines` - The pipelines to convert
/// * `outpoot_root` - The root directory where to save (or find, for validation)
///     the finished Event Logs.
/// * `log_action` - What to do with the mutated event log.
pub fn flattened_pipeline_configs_to_mutation_chains(
    pipelines: Vec<ParametrizedPipelineConfig<Flat>>,
    output_root: &Path,
    log_action: LogAction,
) -> Vec<MutationChain> {
    pipelines
        .into_iter()
        .map(|flat_pipeline_config| {
            flat_pipeline_config.into_mutation_chain(output_root.to_path_buf(), &log_action)
        })
        .collect()
}

impl ParametrizedPipelineConfig<Flat> {
    pub fn into_mutation_chain(
        self,
        mut output_root: PathBuf,
        log_action: &LogAction,
    ) -> MutationChain {
        let mut mutations: Vec<Box<dyn LogMutatorWithAsDirName>> =
            Vec::with_capacity(self.mutations.len());
        // A counter used to create the names for saved event logs.
        // Should be incremented whenever a mutator is created in the pipeline
        // which saves an event log. Used to create unique file names.

        if let Some(seed) = self.seed.clone() {
            output_root.push(format!("{}", seed.inner_value()));
        }

        let mut log_saver_index: u64 = 1;
        self.mutations.into_iter().for_each(|flat_config| {
            let mutator = Self::flat_mutation_config_to_log_mutator(
                flat_config,
                self.seed.clone().map(MutationValue::inner_value),
                output_root.clone(),
                log_action,
                &mut log_saver_index,
            );
            output_root.push(mutator.to_dir_name());
            mutations.push(mutator);
        });
        match log_action {
            LogAction::Save(compress) => {
                let file_name = if log_saver_index == 1 {
                    // No log savers in the pipeline, so log.xes is a unique name
                    "log".into()
                } else {
                    format!("log_{}", log_saver_index)
                };
                // Add an auxilliary mutation which saves the event log
                mutations.push(Box::new(LogSaver::new(
                    build_file_path(output_root, file_name, *compress),
                    *compress,
                )));
            }
            LogAction::Validate(compress) => {
                let file_name = if log_saver_index == 1 {
                    // No log savers in the pipeline, so log.xes is a unique name
                    "log".into()
                } else {
                    format!("log_{}", log_saver_index)
                };
                // Add an auxilliary mutation which validates the event log
                mutations.push(Box::new(LogValidator::new(build_file_path(
                    output_root,
                    file_name,
                    *compress,
                ))));
            }
            LogAction::None => {}
        }
        MutationChain { mutations }
    }

    pub fn with_seed(mut self, seed: u64) -> ParametrizedPipelineConfig<Flat> {
        self.seed = Some(MutationValue::Value(seed));
        self
    }

    fn flat_mutation_config_to_log_mutator(
        flat_config: ParametrizedMutationConfig,
        root_seed: Option<u64>,
        path_so_far: PathBuf,
        log_action: &LogAction,
        log_saver_index: &mut u64,
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
            } => {
                // Convert configuration to a number of seconds
                let days = (365.0 * years.inner_value()) + days.inner_value();
                let hours = (24.0 * days) + hours.inner_value();
                let minutes = (60.0 * hours) + minutes.inner_value();
                let total_seconds = (60.0 * minutes) + seconds.inner_value();
                Box::new(
                    CaseDurationFilter::new(TimeDelta::seconds(total_seconds as i64))
                        .with_sense(sense.inner_value()),
                )
            }
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
            ParametrizedMutationConfig::AttributeRetainer { attributes } => {
                Box::new(AttributeRetainer::new(attributes.inner_value()))
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

                let save_path = save_path.map(|p| {
                    let p = p.inner_value();
                    let save_compressed = save_compressed.clone().map_or_else(
                        || p.extension().is_some_and(|ext| ext == "gz"),
                        MutationValue::inner_value,
                    );
                    ensure_correct_file_extension(p, save_compressed)
                });

                match log_action {
                    LogAction::None => {}
                    LogAction::Save(compress) => {
                        if let Some(p) = save_path {
                            let is_compressed = p.extension().unwrap() == "gz";
                            mutator = mutator.with_save_discarded_log(p, is_compressed);
                        } else {
                            let log_name = format!("log_{}", log_saver_index);
                            let mut path_with_mutator = path_so_far.clone();
                            let save_compressed = save_compressed
                                .clone()
                                .map_or(*compress, MutationValue::inner_value);
                            path_with_mutator.push(mutator.to_dir_name());
                            let save_path =
                                build_file_path(path_with_mutator, log_name, save_compressed);
                            *log_saver_index += 1;

                            mutator = mutator.with_save_discarded_log(save_path, save_compressed);
                        }
                    }
                    LogAction::Validate(compress) => {
                        if let Some(p) = save_path {
                            mutator = mutator.with_validate_discarded_log(p);
                        } else {
                            let log_name = format!("log_{}", log_saver_index);
                            let mut path_with_mutator = path_so_far.clone();
                            let save_compressed = save_compressed
                                .clone()
                                .map_or(*compress, MutationValue::inner_value);
                            path_with_mutator.push(mutator.to_dir_name());
                            let save_path =
                                build_file_path(path_with_mutator, log_name, save_compressed);
                            *log_saver_index += 1;

                            mutator = mutator.with_validate_discarded_log(save_path);
                        }
                    }
                };

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
