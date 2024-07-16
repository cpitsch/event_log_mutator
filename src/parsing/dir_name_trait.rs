pub use derive_macros::DirName;

pub trait DirName {
    fn to_dir_name(&self) -> String;
}
