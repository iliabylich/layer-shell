macro_rules! report_and_exit {
    ($($arg:tt)*) => {{
        let bt = std::backtrace::Backtrace::force_capture();
        log::error!($($arg)*);
        log::error!("Backtrace\n{bt}");
        std::process::exit(1)
    }};
}
pub(crate) use report_and_exit;
