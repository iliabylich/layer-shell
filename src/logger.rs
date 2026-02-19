use log::{Level, LevelFilter, Metadata, Record};
use std::str::FromStr;

use crate::macros::report_and_exit;

pub(crate) struct Logger {
    level: Level,
}

impl Logger {
    pub(crate) fn init() {
        let (level, level_filter) = std::env::var("RUST_LOG")
            .ok()
            .and_then(|v| {
                let level = Level::from_str(&v).ok()?;
                let level_filter = LevelFilter::from_str(&v).ok()?;
                Some((level, level_filter))
            })
            .unwrap_or((Level::Error, LevelFilter::Error));

        let logger: &'static Self = Box::leak(Box::new(Self { level }));
        log::set_logger(logger)
            .map(|()| log::set_max_level(level_filter))
            .unwrap_or_else(|err| report_and_exit!("failed to initialize logger: {err:?}"));
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        dbg!(1);
        const RED: &str = "\x1b[0;31m";
        const GREEN: &str = "\x1b[0;32m";
        const YELLOW: &str = "\x1b[0;33m";
        const WHITE: &str = "\x1b[0;37m";
        const BLUE: &str = "\x1b[0;34m";
        const NC: &str = "\x1b[0m";
        let color = match record.level() {
            Level::Error => RED,
            Level::Warn => YELLOW,
            Level::Info => GREEN,
            Level::Debug | Level::Trace => WHITE,
        };
        if self.enabled(record.metadata()) {
            eprintln!(
                "{color}{}{NC} - {BLUE}{}{NC} - {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}
