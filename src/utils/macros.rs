macro_rules! report_and_exit {
    ($($arg:tt)*) => {{
        let bt = std::backtrace::Backtrace::force_capture();
        log::error!($($arg)*);
        log::error!("Backtrace\n{bt}");
        std::process::exit(1)
    }};
}
pub(crate) use report_and_exit;

macro_rules! assert_or_exit {
    ($cmp:expr, $($arg:tt)*) => {
        if !$cmp {
            $crate::utils::report_and_exit!($($arg)*)
        }
    };
}
pub(crate) use assert_or_exit;
