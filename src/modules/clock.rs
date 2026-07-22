use crate::{
    IoEvent,
    emitter::Emitter,
    utils::{StringRef, StringRefExt, log_err_and_exit},
};
use libc::{localtime_r, strftime, time, tm};

#[derive(Clone, Copy)]
pub struct Clock {
    emitter: Emitter,
}

impl Clock {
    pub(crate) fn new(emitter: Emitter) -> Self {
        log::trace!("Creating Clock");

        Self { emitter }
    }

    pub(crate) fn tick(&self) {
        let now = local_time_string();
        self.emitter.emit(&IoEvent::Time { now });
    }
}

fn local_time_string() -> StringRef {
    unsafe {
        let mut now = 0;
        time(&raw mut now);

        let mut tm: tm = core::mem::zeroed();

        if localtime_r(&raw const now, &raw mut tm).is_null() {
            log_err_and_exit!("failed to localtime()");
        }

        let fmt = c"%H:%M:%S | %b %d | %a";
        let mut buf = [0_u8; 64];

        let n = strftime(
            buf.as_mut_ptr().cast(),
            buf.len(),
            fmt.as_ptr(),
            &raw const tm,
        );

        if n == 0 {
            log_err_and_exit!("failed to strftime()");
        }

        let Some(buf) = buf.get(..n) else {
            log_err_and_exit!("buffer is too short: {} vs {}", buf.len(), n);
        };
        let Ok(time) = core::str::from_utf8(buf) else {
            log_err_and_exit!("non-utf8 strftime result")
        };
        StringRef::new(time)
    }
}
