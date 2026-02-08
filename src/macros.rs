macro_rules! report_and_exit {
    ($($arg:tt)*) => {{
        let bt = std::backtrace::Backtrace::force_capture();
        log::error!($($arg)*);
        log::error!("Backtrace\n{bt}");
        std::process::exit(1)
    }};
}
pub(crate) use report_and_exit;

macro_rules! define_op {
    ($name:expr, $($variant:ident),+ $(,)?) => {
        #[repr(u8)]
        #[derive(Debug)]
        enum Op {
            $($variant),+
        }

        const MAX_OP: u8 = {
            let mut max = 0u8;
            $(
                #[expect(non_snake_case, unused_variables)]
                let $variant = max;
                max += 1;
            )+
            max - 1
        };

        impl From<Op> for u8 {
            fn from(value: Op) -> u8 {
                value as u8
            }
        }

        impl From<u8> for Op {
            fn from(value: u8) -> Self {
                if value > MAX_OP {
                    $crate::macros::report_and_exit!(concat!("unsupported op in ", $name, ": {}"), value)
                }
                unsafe { std::mem::transmute::<u8, Self>(value) }
            }
        }
    };
}
pub(crate) use define_op;

macro_rules! assert_or_exit {
    ($cmp:expr, $($arg:tt)*) => {
        if !$cmp {
            $crate::macros::report_and_exit!($($arg)*)
        }
    };
}
pub(crate) use assert_or_exit;
