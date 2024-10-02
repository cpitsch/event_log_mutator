use std::{
    fs::{create_dir_all, File},
    path::{Path, PathBuf},
};

use process_mining::{event_log::export_xes::export_xes_event_log_to_file, EventLog};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IoError {
    #[error("File {0:?} not found")]
    FileNotFound(PathBuf),
    #[error("The file {0:?} already exists")]
    FileExists(PathBuf),
    #[error("Something went wrong saving the event log {0:?}")]
    XesWriteError(PathBuf),
    #[error("Something went wrong creating the directories on the path {0:?}")]
    DirCreateError(PathBuf),
}

pub fn write_xes(log: &EventLog, path: impl AsRef<Path>, compress: bool) -> Result<(), IoError> {
    let p: &Path = path.as_ref();
    let dir_creation_res = p.parent().map(create_dir_all);
    if dir_creation_res.is_none() || dir_creation_res.unwrap().is_err() {
        return Err(IoError::DirCreateError(p.into()));
    }

    let save_res = File::create(p).map(|file| export_xes_event_log_to_file(log, file, compress));
    if save_res.is_err() || save_res.unwrap().is_err() {
        return Err(IoError::XesWriteError(p.into()));
    }
    Ok(())
}

pub fn build_file_path(base_path: PathBuf, filename: impl Into<String>, compress: bool) -> PathBuf {
    let mut log_path = base_path;
    log_path.push(filename.into());
    log_path = ensure_correct_file_extension(log_path, compress);
    log_path
}

pub fn ensure_correct_file_extension(mut path_to_log: PathBuf, compress: bool) -> PathBuf {
    let extension = if compress { "xes.gz" } else { "xes" };
    if !path_to_log.to_string_lossy().ends_with(extension) {
        while path_to_log.extension().is_some() {
            path_to_log.set_extension("");
        }
        path_to_log.set_extension(extension);
    }
    path_to_log
}
