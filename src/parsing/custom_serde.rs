use crate::parsing::mutation_value::MutationValue;
use serde::{Deserialize, Deserializer};

/// Parse a string of form x..y or x..=y to a Vec of u64
fn parse_u64_range(input: String) -> Result<Vec<u64>, String> {
    let inclusive_range = input.contains("..=");
    if let Some((start, end)) = input.split_once(if inclusive_range { "..=" } else { ".." }) {
        let start = start
            .trim()
            .parse()
            .map_err(|_| ": Could not parse range start".to_string())?;
        let end = end
            .trim()
            .parse()
            .map_err(|_| ": Could not parse range end".to_string())?;
        if end < start {
            Err(": end < start".into())
        } else if inclusive_range {
            Ok((start..=end).collect())
        } else {
            Ok((start..end).collect())
        }
    } else {
        Err("".into())
    }
}

/// Deserialization function to parse MutationValue<u64>, but allow range expressions
pub fn deserialize_u64_vec_or_range<'de, D>(
    deserializer: D,
) -> Result<Option<MutationValue<u64>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum TheVal {
        MutationValue(Option<MutationValue<u64>>),
        RangeExpression(String),
    }

    match TheVal::deserialize(deserializer)? {
        TheVal::MutationValue(mutation_value) => Ok(mutation_value),
        TheVal::RangeExpression(expr) => Ok(Some(MutationValue::Vec(
            parse_u64_range(expr).map_err(|msg| {
                serde::de::Error::custom(format!("Invalid format for range expression{}", msg))
            })?,
        ))),
    }
}
