use process_mining::{event_log::Trace, EventLog};

use crate::{
    constants::NO_COMPLETE_TIMESTAMP_MSG, mutation::LogMutator, parsing::dir_name_trait::DirName,
    utils::attributes::get_complete_timestamp,
};

#[derive(DirName)]
pub struct CaseDurationFilter {
    years: f32,
    days: f32,
    hours: f32,
    minutes: f32,
    seconds: f32,
}

impl CaseDurationFilter {
    pub fn new(
        years: Option<f32>,
        days: Option<f32>,
        hours: Option<f32>,
        minutes: Option<f32>,
        seconds: Option<f32>,
    ) -> Self {
        CaseDurationFilter {
            years: years.unwrap_or(0.0),
            days: days.unwrap_or(0.0),
            hours: hours.unwrap_or(0.0),
            minutes: minutes.unwrap_or(0.0),
            seconds: seconds.unwrap_or(0.0),
        }
    }

    fn keep_trace(&self, trace: &Trace, max_duration: &chrono::TimeDelta) -> bool {
        // Could theoretically use first() and last(), but I don't know for
        // _certain_ that the trace is ordered correctly
        let earliest_timestamp = trace
            .events
            .iter()
            .map(|event| get_complete_timestamp(event).expect(NO_COMPLETE_TIMESTAMP_MSG))
            .min()
            .unwrap();
        let latest_timestamp = trace
            .events
            .iter()
            .map(|event| get_complete_timestamp(event).expect(NO_COMPLETE_TIMESTAMP_MSG))
            .max()
            .unwrap();

        let duration = latest_timestamp - earliest_timestamp;

        duration <= *max_duration
    }

    fn get_total_seconds(&self) -> i64 {
        let days = (365.0 * self.years) + self.days;
        let hours = (24.0 * days) + self.hours;
        let minutes = (60.0 * hours) + self.minutes;
        let seconds = (60.0 * minutes) + self.seconds;

        seconds as i64
    }
}

impl LogMutator for CaseDurationFilter {
    fn apply_mut(&mut self, log: &mut EventLog) {
        let max_duration = chrono::TimeDelta::seconds(self.get_total_seconds());
        log.traces
            .retain(|trace| self.keep_trace(trace, &max_duration));
    }
}
