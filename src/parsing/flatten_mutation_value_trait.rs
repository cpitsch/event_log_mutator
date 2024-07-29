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

    #[derive(Debug, FlattenMutationValue, PartialEq)]
    enum MyEnum {
        Unit,
        Unnamed(MutationValue<bool>, MutationValue<f32>),
        Named {
            field_1: MutationValue<bool>,
            field_2: MutationValue<Option<String>>,
        },
    }

    #[test]
    fn flattens_unit_enum_variant() {
        let test_instance = MyEnum::Unit;
        assert_eq!(test_instance.flatten(), vec![MyEnum::Unit]);
    }

    #[test]
    fn flattens_unnamed_variant() {
        let test_instance = MyEnum::Unnamed(
            MutationValue::Vec(vec![true, false]),
            MutationValue::Value(0.0),
        );

        assert_eq!(
            test_instance.flatten(),
            vec![
                MyEnum::Unnamed(MutationValue::Value(true), MutationValue::Value(0.0)),
                MyEnum::Unnamed(MutationValue::Value(false), MutationValue::Value(0.0)),
            ]
        );
    }

    #[test]
    fn flattens_named_variant() {
        let test_instance = MyEnum::Named {
            field_1: MutationValue::Value(true),
            field_2: MutationValue::Vec(vec![None, Some("Hello, World!".into())]),
        };

        assert_eq!(
            test_instance.flatten(),
            vec![
                MyEnum::Named {
                    field_1: MutationValue::Value(true),
                    field_2: MutationValue::Value(None)
                },
                MyEnum::Named {
                    field_1: MutationValue::Value(true),
                    field_2: MutationValue::Value(Some("Hello, World!".into()))
                }
            ]
        );
    }

    #[test]
    fn flattens_structs() {
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
