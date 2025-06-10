use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum MutationValue<T> {
    Value(T),
    Vec(Vec<T>),
}

impl<T> MutationValue<T> {
    pub fn get_as_vec(self) -> Vec<T> {
        match self {
            Self::Vec(v) => v,
            Self::Value(v) => vec![v],
        }
    }

    pub fn inner_value(self) -> T {
        match self {
            Self::Value(v) => v,
            Self::Vec(_) => panic!("Called inner_value on non-flat MutationValue"),
        }
    }
}

impl<T> Default for MutationValue<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::Value(T::default())
    }
}
