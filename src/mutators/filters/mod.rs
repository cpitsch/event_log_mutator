pub mod attribute_value_filter;
pub mod case_duration_filter;
pub mod endpoint_filter;
pub mod follower_filter;
pub mod trace_length_filter;
pub mod variant_support_filter;

use std::fmt::Display;

pub use attribute_value_filter::AttributeFilter;
pub use case_duration_filter::CaseDurationFilter;
pub use endpoint_filter::EndpointFilter;
pub use follower_filter::FollowerFilter;
use serde::Deserialize;
pub use trace_length_filter::TraceLengthFilter;
pub use variant_support_filter::VariantSupportFilter;

#[derive(Deserialize, Debug, Default, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ComparisonSense {
    #[serde(alias = "less", alias = "<")]
    Less,
    #[default]
    #[serde(alias = "leq", alias = "<=")]
    LEQ,
    #[serde(alias = "geq", alias = ">=")]
    GEQ,
    #[serde(alias = "greater", alias = ">")]
    Greater,
}

impl ComparisonSense {
    pub fn compare<T>(&self, first: &T, other: &T) -> bool
    where
        T: PartialOrd,
    {
        match self {
            Self::Less => first < other,
            Self::LEQ => first <= other,
            Self::Greater => first > other,
            Self::GEQ => first >= other,
        }
    }
}

impl Display for ComparisonSense {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Less => "less",
                Self::LEQ => "leq",
                Self::GEQ => "geq",
                Self::Greater => "greater",
            }
        )
    }
}
