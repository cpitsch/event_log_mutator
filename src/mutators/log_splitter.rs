use process_mining::EventLog;
use rand::{rngs::StdRng, SeedableRng};

use crate::{
    mutation::LogMutator,
    parsing::dir_name_trait::DirName,
    utils::{get_traceid, get_traceids, sample_log_without_replacement},
    write_xes,
};

#[derive(DirName)]
pub struct LogSplitter {
    frac: f64,
    #[dirname(ignore)]
    save_path: Option<String>,
    #[dirname(ignore)]
    save_compressed: bool,
    seed: Option<u64>,
    #[dirname(ignore)]
    rng: StdRng,
}

impl LogSplitter {
    pub fn new(frac: f64) -> Self {
        if frac > 1.0 {
            panic!("LogSplitter cannot be used with a sampling fraction greater than 1");
        }

        Self {
            frac,
            save_path: None,
            save_compressed: false,
            seed: None,
            rng: StdRng::from_entropy(),
        }
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self.rng = StdRng::seed_from_u64(seed);
        self
    }

    pub fn save_discarded(mut self, path: String) -> Self {
        self.save_path = Some(path);
        self
    }

    pub fn save_compressed(mut self, compressed: bool) -> Self {
        self.save_compressed = compressed;
        self
    }
}

impl LogMutator for LogSplitter {
    fn apply(&mut self, log: &EventLog) -> EventLog {
        let size = ((log.traces.len() as f64) * self.frac).round() as usize;
        let new_log = sample_log_without_replacement(&mut self.rng, log, size);
        if let Some(path) = self.save_path.clone() {
            let trace_ids = get_traceids(&new_log);
            let mut complement = log.clone();
            complement
                .traces
                .retain(|trace| !trace_ids.contains(&get_traceid(trace).unwrap()));
            write_xes(&complement, path, self.save_compressed).unwrap();
        }
        new_log
    }
}
