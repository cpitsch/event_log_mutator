use clap::ValueEnum;
use process_mining::EventLog;

use crate::{
    cli::Args,
    mutation::MutationChain,
    mutators::{
        filters::VariantSupportFilter, ActivityRemover, EventSwapper, LogBootstrapper,
        PartialOrderCreator, ServiceTimeMultiplier,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Preset {
    /// Bootstrap a "new" event log of the same size by sampling cases with replacement
    Bootstrap,
    /// Turn an atomic event log into a partially ordered event log by using the
    /// time since the previous event as the service time
    PartialOrder,
    Bpi12OnlyServiceTime,
    Bpi12,
    /// Bootstrap, then multiply the service time of "Send Fine" by 2
    RoadTraffic,
    /// Bootstrap, then swap events "Send Fine" and "Payment"
    RoadTrafficSwap,
    /// Retain only the cases whose variant is supported by at least `n` cases total
    FilterVariantSupport,
}

impl Preset {
    pub fn into_mutation_chain(self, log: &EventLog, args: Args) -> MutationChain {
        match self {
            Self::Bootstrap => {
                MutationChain::new().with_mutation(LogBootstrapper::new(log.traces.len()))
            }
            Self::PartialOrder => MutationChain::new().with_mutation(PartialOrderCreator::new()),
            Self::Bpi12 => MutationChain::new()
                .with_mutation(LogBootstrapper::new(log.traces.len()))
                .with_mutation(
                    ServiceTimeMultiplier::new(2.0)
                        .for_activity("W_Completeren aanvraag")
                        .with_probability(1.0),
                )
                .with_mutation(
                    // Only 270 instances in the original log
                    ActivityRemover::new("W_Beoordelen fraude").with_probability(1.0),
                ),
            Self::Bpi12OnlyServiceTime => MutationChain::new()
                .with_mutation(LogBootstrapper::new(log.traces.len()))
                .with_mutation(
                    ServiceTimeMultiplier::new(2.0)
                        .for_activity("W_Completeren aanvraag")
                        .with_probability(1.0),
                ),
            Self::RoadTraffic => MutationChain::new()
                .with_mutation(LogBootstrapper::new(log.traces.len()))
                .with_mutation(
                    ServiceTimeMultiplier::new(args.severity.unwrap_or(2.0))
                        .for_activity("Send Fine")
                        .with_probability(args.probability.unwrap_or(1.0)),
                ),
            // .with_mutation(
            //     ServiceTimeMultiplier::new(2.0)
            //         .for_activity("Send for Credit Collection"),
            // );
            Self::RoadTrafficSwap => MutationChain::new()
                .with_mutation(LogBootstrapper::new(log.traces.len()))
                .with_mutation(EventSwapper::new("Send Fine", "Payment")),
            Self::FilterVariantSupport => {
                MutationChain::new()
                    .with_mutation(VariantSupportFilter::new(args.support.expect(
                        "Variant Support Filter requires the `--support` flag to be set.",
                    )))
            }
        }
    }
}
