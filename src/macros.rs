macro_rules! fatal {
    ($($arg:tt)+) => {{
        log::error!($($arg)+);
        std::process::exit(1);
    }}
}
pub(crate) use fatal;

macro_rules! cast_ctx_ref {
    ($ctx:ident, $t:ty) => {{
        let $ctx = unsafe { $ctx.cast::<$t>().as_ref() };
        $ctx.unwrap_or_else(|| $crate::macros::fatal!("Can't read NULL ctx"))
    }};
}
pub(crate) use cast_ctx_ref;

macro_rules! cast_ctx_mut {
    ($ctx:ident, $t:ty) => {{
        let $ctx = unsafe { $ctx.cast::<$t>().as_mut() };
        $ctx.unwrap_or_else(|| $crate::macros::fatal!("Can't read NULL ctx"))
    }};
}
pub(crate) use cast_ctx_mut;
