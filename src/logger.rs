use std::str::FromStr;

pub(crate) struct StdErrLogger {
    level: log::LevelFilter,
}

impl StdErrLogger {
    pub(crate) fn new() -> Self {
        let level = std::env::var("RUST_LOG")
            .ok()
            .and_then(|var| log::LevelFilter::from_str(&var).ok())
            .unwrap_or(log::LevelFilter::Error);
        Self { level }
    }
}

impl log::Log for StdErrLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let target = if !record.target().is_empty() {
                record.target()
            } else {
                record.module_path().unwrap_or_default()
            };

            let line = record.line().unwrap_or(0);
            let args = record.args();

            eprintln!(
                "{} (in {}:{}): {}",
                record.level().as_str(),
                target,
                line,
                args
            );
        }
    }

    fn flush(&self) {}
}
