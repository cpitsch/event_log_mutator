use std::path::PathBuf;

use colored::Colorize;
use log::{info, warn};
use process_mining::core::event_data::case_centric::{
    xes::{import_xes_path, XESImportOptions},
    EventLog,
};

use crate::{
    mutation::{LogMutator, MutationResult},
    parsing::traits::DirName,
    utils::compare::event_logs_are_identical,
};

/// Auxilliary mutator to take an event log and compare it to the one stored
/// at a certain path
#[derive(DirName)]
pub struct LogValidator {
    #[dirname(ignore)] // Having a path in the path would be weird
    path: PathBuf,
}

impl LogValidator {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl LogMutator for LogValidator {
    fn apply_mut(&mut self, log: &mut EventLog) -> MutationResult<()> {
        if !self.path.exists() {
            warn!("Event Log does not exist: {}", self.path.to_string_lossy());
        } else {
            let valid = event_logs_are_identical(
                log,
                &import_xes_path(&self.path, XESImportOptions::default())?,
            );
            if !valid {
                warn!("Event Log mismatch: {}", self.path.to_string_lossy());
            } else {
                let ok = "OK".green();
                info!("{} {}", ok, self.path.to_string_lossy());
            }
        }
        Ok(())
    }
}
