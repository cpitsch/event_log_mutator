pub use derive_macros::FlattenMutationValue;

pub trait FlattenMutationValue
where
    Self: std::marker::Sized,
{
    fn flatten(self) -> Vec<Self>;
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parsing::parametrized_pipeline::MutationValue;

    #[derive(Debug, FlattenMutationValue, PartialEq)]
    struct MyStruct {
        field_1: MutationValue<bool>,
        field_2: MutationValue<f32>,
        field_3: MutationValue<Option<String>>,
    }

    #[test]
    fn flattens_correctly() {
        let test_instance = MyStruct {
            field_1: MutationValue::Vec(vec![true, false]),
            field_2: MutationValue::Vec(vec![0.0]), // Vec with a single value
            field_3: MutationValue::Value(None),
        };

        assert_eq!(
            test_instance.flatten(),
            vec![
                MyStruct {
                    field_1: MutationValue::Value(true),
                    field_2: MutationValue::Value(0.0),
                    field_3: MutationValue::Value(None)
                },
                MyStruct {
                    field_1: MutationValue::Value(false),
                    field_2: MutationValue::Value(0.0),
                    field_3: MutationValue::Value(None)
                },
            ]
        )
    }
}
