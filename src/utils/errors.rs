use std::error::Error;

/// [Vec::retain], but the filter function can error. In this case, the error is propagated
/// upwards and the vec remains untouched.
// TODO: Could make a RetainErr trait?
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
