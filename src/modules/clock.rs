use crate::{
    IoEvent,
    emitter::Emitter,
    error::IoError,
    utils::{StringRef, StringRefExt},
};
use libc::{localtime_r, strftime, time, tm};

#[derive(Clone, Copy)]
pub(crate) struct Clock {
    emitter: Emitter,
}

impl Clock {
    pub(crate) const fn new(emitter: Emitter) -> Self {
        Self { emitter }
    }

    pub(crate) fn tick(&self) -> Result<(), IoError> {
        let now = local_time_string()?;
        self.emitter.emit(&IoEvent::Time { now });
        Ok(())
    }
}

fn local_time_string() -> Result<StringRef, IoError> {
    unsafe {
        let mut now = 0;
        time(&raw mut now);

        let mut tm: tm = core::mem::zeroed();

        if localtime_r(&raw const now, &raw mut tm).is_null() {
            return Err(IoError::new_failed_to("localtime"));
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
            return Err(IoError::new_failed_to("strftime"));
        }

        let Some(buf) = buf.get(..n) else {
            unreachable!("buffer is too short: {} vs {}", buf.len(), n);
        };
        let Ok(time) = core::str::from_utf8(buf) else {
            unreachable!("non-utf8 strftime result")
        };
        Ok(StringRef::new(time))
    }
}
