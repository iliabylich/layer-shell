use crate::utils::{ArrayWriter, getenv};
use anyhow::{Context as _, Result};
use core::{fmt::Write as _, str::FromStr as _};
use log::{LevelFilter, Metadata, Record};

static LOGGER: Logger = Logger;

pub(crate) fn init() -> Result<()> {
    if log::set_logger(&LOGGER).is_ok() {
        log::set_max_level(level_from_env()?);
    }
    Ok(())
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let mut buf = [0; 4_096];
        let mut writer = ArrayWriter::new(&mut buf);

        let res = writeln!(
            writer,
            "{}{}{} {}{}{}: {}",
            color_for_level(record.level()),
            record.level(),
            RESET,
            GRAY,
            record.target(),
            RESET,
            record.args()
        );

        let bytes = if res.is_ok() {
            writer
                .buf
                .get(..writer.offset)
                .unwrap_or(b"ERROR logger: malformed log buffer\n")
        } else {
            b"ERROR logger: log message does not fit into buffer\n"
        };

        unsafe {
            libc::write(libc::STDERR_FILENO, bytes.as_ptr().cast(), bytes.len());
        }
    }

    fn flush(&self) {}
}

fn level_from_env() -> Result<LevelFilter> {
    let Some(level) = getenv(c"RUST_LOG") else {
        return Ok(LevelFilter::Info);
    };

    let level = core::str::from_utf8(level).context("non-utf8 $RUST_LOG")?;
    LevelFilter::from_str(level)
        .map_err(|_| anyhow::anyhow!("unsupported $RUST_LOG value: {level}"))
}

const RESET: &str = "\x1b[0m";
const GRAY: &str = "\x1b[2m";
const RED: &str = "\x1b[31;1m";
const YELLOW: &str = "\x1b[33;1m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";

const fn color_for_level(level: log::Level) -> &'static str {
    match level {
        log::Level::Error => RED,
        log::Level::Warn => YELLOW,
        log::Level::Info => GREEN,
        log::Level::Debug => CYAN,
        log::Level::Trace => MAGENTA,
    }
}
