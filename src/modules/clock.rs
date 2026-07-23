use crate::{
    IoEvent,
    emitter::Emitter,
    utils::{StringRef, StringRefExt},
};
use libc::{localtime_r, strftime, time, tm};

pub struct Clock;

impl Clock {
    pub(crate) fn tick(emitter: Emitter) {
        let now = local_time_string();
        emitter.emit(&IoEvent::Time { now });
    }
}

fn local_time_string() -> StringRef {
    unsafe {
        let mut now = 0;
        time(&raw mut now);

        let mut tm: tm = core::mem::zeroed();

        let res = localtime_r(&raw const now, &raw mut tm);
        assert!(!res.is_null(), "failed to localtime()");

        let fmt = c"%H:%M:%S | %b %d | %a";
        let mut buf = [0_u8; 64];

        let n = strftime(
            buf.as_mut_ptr().cast(),
            buf.len(),
            fmt.as_ptr(),
            &raw const tm,
        );

        assert!(n > 0, "failed to strftime()");

        let Some(buf) = buf.get(..n) else {
            panic!("buffer is too short: {} vs {}", buf.len(), n);
        };
        let Ok(time) = core::str::from_utf8(buf) else {
            panic!("non-utf8 strftime result")
        };
        StringRef::new(time)
    }
}
