pub mod activity_remover;
pub mod activity_renamer;
pub mod attribute_remover;
pub mod attribute_retainer;
pub mod constant_activity;
pub mod event_swapper;
pub mod log_bootstrapper;
pub mod log_splitter;
pub mod partial_order_creator;
pub mod service_time_adder;
pub mod service_time_multiplier;
pub mod service_time_std_shifter;

pub mod aux_mutators;
pub mod filters;

use itertools::Itertools;
use std::{fmt::Display, ops::Deref};

pub use activity_remover::ActivityRemover;
pub use activity_renamer::ActivityRenamer;
pub use attribute_remover::AttributeRemover;
pub use attribute_retainer::AttributeRetainer;
pub use constant_activity::ConstantActivityMutator;
pub use event_swapper::EventSwapper;
pub use log_bootstrapper::LogBootstrapper;
pub use log_splitter::LogSplitter;
pub use partial_order_creator::PartialOrderCreator;
pub use service_time_adder::ServiceTimeAdder;
pub use service_time_multiplier::ServiceTimeMultiplier;
pub use service_time_std_shifter::ServiceTimeStdShifter;

#[derive(Clone)]
pub struct DisplayVec<T: Display>(Vec<T>);

impl<T: Display> Display for DisplayVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().join("+"))
    }
}

impl<T> From<DisplayVec<T>> for Vec<T>
where
    T: Display,
{
    fn from(val: DisplayVec<T>) -> Self {
        val.0
    }
}

impl<T> From<Vec<T>> for DisplayVec<T>
where
    T: Display,
{
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

impl<T> Deref for DisplayVec<T>
where
    T: Display,
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
