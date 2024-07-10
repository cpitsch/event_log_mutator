pub use derive_macros::AsDirName;

pub trait AsDirName {
    fn as_dir_name(&self) -> String;
}
