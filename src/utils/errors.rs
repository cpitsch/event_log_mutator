use std::error::Error;

pub fn retain_err<T, F, E>(vec: &mut Vec<T>, f: F) -> Result<(), E>
where
    F: FnMut(&T) -> Result<bool, E>,
    E: Error,
{
    let mut retain_indices = vec
        .iter()
        .map(f)
        .collect::<Result<Vec<bool>, E>>()?
        .into_iter();

    vec.retain(|_| retain_indices.next().unwrap());

    Ok(())
}
