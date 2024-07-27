pub use derive_macros::FlattenMutationValue;

pub trait FlattenMutationValue
where
    Self: std::marker::Sized,
{
    fn flatten(self) -> Vec<Self>;
}
