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
pub fn deserialize_u64_vec_or_range<'de, D>(deserializer: D) -> Result<MutationValue<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum TheValue {
        MutationValue(MutationValue<u64>),
        RangeExpression(String),
    }

    match TheValue::deserialize(deserializer)? {
        TheValue::MutationValue(mutation_value) => Ok(mutation_value),
        TheValue::RangeExpression(expr) => Ok(MutationValue::Vec(parse_u64_range(expr).map_err(
            |msg| serde::de::Error::custom(format!("Invalid format for range expression {msg}")),
        )?)),
    }
}

pub fn deserialize_u64_vec_or_range_option<'de, D>(
    deserializer: D,
) -> Result<Option<MutationValue<u64>>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Some(deserialize_u64_vec_or_range(deserializer)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::mutation_value::MutationValue;
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct TestStruct {
        #[serde(deserialize_with = "deserialize_u64_vec_or_range_option")]
        pub my_field: Option<MutationValue<u64>>,
    }

    #[test]
    fn test_inclusive_range_expr() {
        let expr = "1..=25";
        let res: TestStruct = toml::from_str(format!("my_field=\"{}\"", expr).as_str()).unwrap();
        let expected: Vec<u64> = (1..=25).collect();
        assert!(res.my_field.is_some_and(|val| val.get_as_vec() == expected));
    }

    #[test]
    fn test_non_inclusive_range_expr() {
        let expr = "1..25";
        let res: TestStruct = toml::from_str(format!("my_field=\"{}\"", expr).as_str()).unwrap();
        let expected: Vec<u64> = (1..25).collect();
        assert!(res.my_field.is_some_and(|val| val.get_as_vec() == expected));
    }

    #[test]
    fn test_lb_greater_than_ub() {
        let expr = "25..1";
        let res = toml::from_str::<TestStruct>(format!("my_field=\"{}\"", expr).as_str());
        assert!(res.is_err());
    }

    #[test]
    fn test_using_floats() {
        let expr = "1..25.7";
        let res = toml::from_str::<TestStruct>(format!("my_field=\"{}\"", expr).as_str());
        assert!(res.is_err());
    }

    #[test]
    fn test_single_value() {
        let expr = 7;
        let res: TestStruct = toml::from_str(format!("my_field={}", expr).as_str()).unwrap();
        assert!(res.my_field.is_some_and(|val| val.inner_value() == expr));
    }

    #[test]
    fn test_vec() {
        let expr = "[2,4,6,8]";
        let res: TestStruct = toml::from_str(format!("my_field={}", expr).as_str()).unwrap();
        let expected: Vec<u64> = vec![2, 4, 6, 8];
        assert!(res.my_field.is_some_and(|val| val.get_as_vec() == expected));
    }

    #[test]
    fn non_range_expr_string_errors() {
        let expr = "hello, world!";
        let res = toml::from_str::<TestStruct>(format!("my_field=\"{}\"", expr).as_str());
        assert!(res.is_err());
    }
}
