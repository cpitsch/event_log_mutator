use std::path::PathBuf;

use process_mining::EventLog;

use crate::{mutation::LogMutator, parsing::traits::DirName, utils::io::write_xes};

#[derive(DirName)]
pub struct LogSaver {
    #[dirname(ignore)] // Having a path in the path would be weird
    path: PathBuf,
    compress: bool,
}

impl LogSaver {
    pub fn new(path: PathBuf, compress: bool) -> Self {
        Self { path, compress }
    }
}

impl LogMutator for LogSaver {
    fn apply_mut(&mut self, log: &mut EventLog) {
        write_xes(log, self.path.as_path(), self.compress).unwrap()
    }
}
