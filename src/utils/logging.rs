use chrono::Local;
use colored::{ColoredString, Colorize};
use log::{Level, LevelFilter};
use std::{io::Write, u8};

fn color_logging_string(log_str: impl ToString, level: Level) -> ColoredString {
    match level {
        Level::Info => log_str.to_string().cyan(),
        Level::Warn => log_str.to_string().yellow(),
        Level::Error => log_str.to_string().red(),
        Level::Debug => log_str.to_string().bright_green(),
        Level::Trace => log_str.to_string().white(),
    }
}

pub fn verbosity_to_level_filter(verbosity: u8) -> LevelFilter {
    match verbosity {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        5..=u8::MAX => LevelFilter::Trace,
    }
}

pub fn init_logger(verbose: u8, quiet: u8) {
    let total_verbosity = verbose.saturating_sub(quiet);
    env_logger::Builder::new()
        .filter_level(verbosity_to_level_filter(total_verbosity))
        .format(|buf, record| {
            let level = record.level();
            writeln!(
                buf,
                "{}{} {} {}{} {}",
                "[".bright_black(),
                color_logging_string(level, level),
                "-".bright_black(),
                color_logging_string(Local::now().format("%H:%M:%S"), level),
                "]".bright_black(),
                record.args()
            )
        })
        .init();
}
