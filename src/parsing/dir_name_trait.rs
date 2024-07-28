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

    // #[derive(DirName)]
    // struct UnitStruct;

    // Currently would fail due to unwrapping the ident of the unnamed fields
    // #[derive(DirName)]
    // struct TupleStruct(i32, bool);

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

    // #[test]
    // fn for_unit_struct() {
    //     let test_instance = UnitStruct {};
    //
    //     // Currently fails due to trailing underscore
    //     assert_eq!(test_instance.to_dir_name(), "UnitStruct".to_string())
    // }

    // #[test]
    // fn for_tuple_sruct() {
    //     let test_instance = TupleStruct(1, true);
    //
    //     // Not implemented yet, but would probably look like this:
    //     // Or booleans could be turned into "yes" or "no"
    //     assert_eq!(test_instance.to_dir_name(), "TupleStruct_1_true");
    // }
}
