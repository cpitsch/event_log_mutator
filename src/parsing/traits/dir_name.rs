pub use derive_macros::DirName;

pub trait DirName {
    fn to_dir_name(&self) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(DirName)]
    struct MyStruct {
        field_1: bool,
        field_2: f32,
        field_3: Option<String>,
    }

    #[derive(DirName)]
    struct UnitStruct;

    #[derive(DirName)]
    struct TupleStruct(i32, bool);

    #[derive(DirName)]
    struct TupleStructWithOption(Option<String>, bool);

    #[derive(DirName)]
    struct StructWithIgnoreField {
        field_1: bool,
        field_2: f32,
        #[dirname(ignore)]
        #[allow(dead_code)]
        hidden_field: String,
    }

    #[test]
    fn for_true_and_some() {
        let test_instance = MyStruct {
            field_1: true,
            field_2: 1.0,
            field_3: Some("val".into()),
        };

        assert_eq!(
            test_instance.to_dir_name(),
            "MyStruct_field_1_field_2_1_field_3_val".to_string()
        );
    }

    #[test]
    fn for_false_and_none() {
        let test_instance = MyStruct {
            field_1: false,
            field_2: 1.0,
            field_3: None,
        };

        assert_eq!(
            test_instance.to_dir_name(),
            "MyStruct_no_field_1_field_2_1_No_field_3".to_string()
        );
    }

    #[test]
    fn for_unit_struct() {
        let test_instance = UnitStruct {};
        assert_eq!(test_instance.to_dir_name(), "UnitStruct".to_string())
    }

    #[test]
    fn for_tuple_struct() {
        let test_instance = TupleStruct(1, true);

        assert_eq!(test_instance.to_dir_name(), "TupleStruct_1_true");
    }

    #[test]
    fn for_tuple_struct_with_some_option() {
        let test_instance = TupleStructWithOption(Some("TEST".to_string()), false);

        assert_eq!(
            test_instance.to_dir_name(),
            "TupleStructWithOption_TEST_false"
        );
    }
    #[test]
    fn for_tuple_struct_with_none_option() {
        let test_instance = TupleStructWithOption(None, false);

        assert_eq!(
            test_instance.to_dir_name(),
            "TupleStructWithOption_None_false"
        );
    }

    #[test]
    fn ignored_field_is_ignored() {
        let test_instance = StructWithIgnoreField {
            field_1: true,
            field_2: 1.0,
            hidden_field: "This shouldn't show".into(),
        };

        assert_eq!(
            test_instance.to_dir_name(),
            "StructWithIgnoreField_field_1_field_2_1"
        )
    }
}
