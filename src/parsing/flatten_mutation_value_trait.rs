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

    #[derive(Debug, FlattenMutationValue, PartialEq)]
    enum EnumWithOptionalsAndOthers {
        Unnamed(MutationValue<bool>, Option<MutationValue<u32>>, bool),
        Named {
            field_1: MutationValue<bool>,
            field_2: Option<MutationValue<u32>>,
            field_3: bool,
        },
    }

    #[derive(Debug, FlattenMutationValue, PartialEq)]
    struct StructWithOptionalsAndOthers {
        field_1: MutationValue<bool>,
        field_2: Option<MutationValue<f32>>,
        field_3: usize,
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
    fn flattens_named_enum_with_others_and_optional() {
        let test_instance_1 = EnumWithOptionalsAndOthers::Named {
            field_1: MutationValue::Vec(vec![true, false]),
            field_2: None,
            field_3: false,
        };

        assert_eq!(
            test_instance_1.flatten(),
            vec![
                EnumWithOptionalsAndOthers::Named {
                    field_1: MutationValue::Value(true),
                    field_2: None,
                    field_3: false
                },
                EnumWithOptionalsAndOthers::Named {
                    field_1: MutationValue::Value(false),
                    field_2: None,
                    field_3: false
                }
            ]
        );

        let test_instance_2 = EnumWithOptionalsAndOthers::Named {
            field_1: MutationValue::Vec(vec![true, false]),
            field_2: Some(MutationValue::Vec(vec![1, 2])),
            field_3: false,
        };

        assert_eq!(
            test_instance_2.flatten(),
            vec![
                EnumWithOptionalsAndOthers::Named {
                    field_1: MutationValue::Value(true),
                    field_2: Some(MutationValue::Value(1)),
                    field_3: false
                },
                EnumWithOptionalsAndOthers::Named {
                    field_1: MutationValue::Value(true),
                    field_2: Some(MutationValue::Value(2)),
                    field_3: false
                },
                EnumWithOptionalsAndOthers::Named {
                    field_1: MutationValue::Value(false),
                    field_2: Some(MutationValue::Value(1)),
                    field_3: false
                },
                EnumWithOptionalsAndOthers::Named {
                    field_1: MutationValue::Value(false),
                    field_2: Some(MutationValue::Value(2)),
                    field_3: false
                },
            ]
        );
    }

    #[test]
    fn flattens_unnamed_enum_with_others_and_optional() {
        let test_instance_1 =
            EnumWithOptionalsAndOthers::Unnamed(MutationValue::Vec(vec![true, false]), None, false);
        assert_eq!(
            test_instance_1.flatten(),
            vec![
                EnumWithOptionalsAndOthers::Unnamed(MutationValue::Value(true), None, false),
                EnumWithOptionalsAndOthers::Unnamed(MutationValue::Value(false), None, false),
            ]
        );

        let test_instance_2 = EnumWithOptionalsAndOthers::Unnamed(
            MutationValue::Vec(vec![true, false]),
            Some(MutationValue::Vec(vec![1, 2])),
            true,
        );

        assert_eq!(
            test_instance_2.flatten(),
            vec![
                EnumWithOptionalsAndOthers::Unnamed(
                    MutationValue::Value(true),
                    Some(MutationValue::Value(1)),
                    true
                ),
                EnumWithOptionalsAndOthers::Unnamed(
                    MutationValue::Value(true),
                    Some(MutationValue::Value(2)),
                    true
                ),
                EnumWithOptionalsAndOthers::Unnamed(
                    MutationValue::Value(false),
                    Some(MutationValue::Value(1)),
                    true
                ),
                EnumWithOptionalsAndOthers::Unnamed(
                    MutationValue::Value(false),
                    Some(MutationValue::Value(2)),
                    true
                ),
            ]
        )
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

    #[test]
    fn flattens_struct_with_others_and_optional() {
        let test_instance_1 = StructWithOptionalsAndOthers {
            field_1: MutationValue::Vec(vec![true, false]),
            field_2: None,
            field_3: 1,
        };

        assert_eq!(
            test_instance_1.flatten(),
            vec![
                StructWithOptionalsAndOthers {
                    field_1: MutationValue::Value(true),
                    field_2: None,
                    field_3: 1
                },
                StructWithOptionalsAndOthers {
                    field_1: MutationValue::Value(false),
                    field_2: None,
                    field_3: 1
                },
            ]
        );

        let test_instance_2 = StructWithOptionalsAndOthers {
            field_1: MutationValue::Vec(vec![true, false]),
            field_2: Some(MutationValue::Vec(vec![1.0, 2.0])),
            field_3: 1,
        };

        assert_eq!(
            test_instance_2.flatten(),
            vec![
                StructWithOptionalsAndOthers {
                    field_1: MutationValue::Value(true),
                    field_2: Some(MutationValue::Value(1.0)),
                    field_3: 1
                },
                StructWithOptionalsAndOthers {
                    field_1: MutationValue::Value(true),
                    field_2: Some(MutationValue::Value(2.0)),
                    field_3: 1
                },
                StructWithOptionalsAndOthers {
                    field_1: MutationValue::Value(false),
                    field_2: Some(MutationValue::Value(1.0)),
                    field_3: 1
                },
                StructWithOptionalsAndOthers {
                    field_1: MutationValue::Value(false),
                    field_2: Some(MutationValue::Value(2.0)),
                    field_3: 1
                },
            ]
        );
    }
}
