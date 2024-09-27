use std::{
    fs::{create_dir_all, File},
    path::{Path, PathBuf},
};

use clap::error::ErrorKind;
use process_mining::{event_log::export_xes::export_xes_event_log_to_file, EventLog};

use crate::cli::CliError;

pub fn write_xes(log: &EventLog, path: impl AsRef<Path>, compress: bool) -> Result<(), CliError> {
    let p: &Path = path.as_ref();
    let dir_creation_res = p.parent().map(create_dir_all);
    if dir_creation_res.is_none() || dir_creation_res.unwrap().is_err() {
        return Err(CliError::new(
            ErrorKind::Io,
            format!(
                "Something went wrong creating the directories on the path {}",
                p.to_string_lossy()
            ),
        ));
    }

    let save_res = File::create(p).map(|file| export_xes_event_log_to_file(log, file, compress));
    if save_res.is_err() || save_res.unwrap().is_err() {
        return Err(CliError::new(
            ErrorKind::Io,
            "Something went wrong while saving the file.",
        ));
    }
    Ok(())
}

pub fn build_file_path(base_path: PathBuf, filename: impl Into<String>, compress: bool) -> PathBuf {
    let mut log_path = base_path;
    log_path.push(filename.into());
    if compress {
        log_path.set_extension("xes.gz");
    } else {
        log_path.set_extension("xes");
    }
    log_path
}
