use std::path::PathBuf;

use process_mining::EventLog;
use rand::{rngs::StdRng, SeedableRng};

use crate::{
    mutation::{LogMutator, MutationError, MutationResult},
    parsing::traits::DirName,
    utils::{
        attributes::{get_traceid, get_traceids, AttributeResult},
        errors::retain_err,
        io::{ensure_correct_file_extension, write_xes},
        sampling::sample_log_without_replacement,
    },
};

#[derive(DirName)]
pub struct LogSplitter {
    frac: f64,
    seed: Option<u64>,
    #[dirname(ignore)]
    rng: StdRng,
    #[dirname(ignore)]
    handle_discarded_log: Option<Box<dyn Fn(EventLog) -> MutationResult<()>>>,
}

impl LogSplitter {
    pub fn new(frac: f64) -> Self {
        if frac > 1.0 {
            panic!("LogSplitter cannot be used with a sampling fraction greater than 1");
        }

        Self {
            frac,
            seed: None,
            rng: StdRng::from_entropy(),
            handle_discarded_log: None,
        }
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self.rng = StdRng::seed_from_u64(seed);
        self
    }

    pub fn with_discard_handler(mut self, f: Box<dyn Fn(EventLog) -> MutationResult<()>>) -> Self {
        self.handle_discarded_log = Some(Box::new(f));
        self
    }

    pub fn with_save_discarded_log(self, save_path: PathBuf, compress: bool) -> Self {
        self.with_discard_handler(Box::new(move |log| {
            let path = ensure_correct_file_extension(save_path.clone(), compress);
            Ok(write_xes(&log, path, compress)?)
        }))
    }
}

impl LogMutator for LogSplitter {
    fn apply_mut(&mut self, log: &mut EventLog) -> MutationResult<()> {
        let size = ((log.traces.len() as f64) * self.frac).round() as usize;
        let discarded_log = sample_log_without_replacement(&mut self.rng, log, size)?;
        let discarded_traceids = get_traceids(&discarded_log)
            .map_err(|e| MutationError::MissingAttributeError("LogSplitter", e))?;

        retain_err(&mut log.traces, |trace| -> AttributeResult<bool> {
            let traceid = get_traceid(trace)?;
            Ok(!discarded_traceids.contains(&traceid))
        })
        .map_err(|e| MutationError::MissingAttributeError("LogSplitter", e))?;

        if let Some(f) = self.handle_discarded_log.as_ref() {
            f(discarded_log)?;
        }

        Ok(())
    }
}
