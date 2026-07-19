use crate::{
    Event,
    emitter::Emitter,
    utils::{StringRef, StringRefExt},
};
use alloc::string::String;
use anyhow::{Result, bail};
use libc::{localtime_r, strftime, time, tm};

pub(crate) struct Clock {
    emitter: Emitter,
}

impl Clock {
    pub(crate) const fn new(emitter: Emitter) -> Self {
        Self { emitter }
    }

    pub(crate) fn tick(&self) -> Result<()> {
        let now = StringRef::new(&local_time_string()?);
        self.emitter.emit(&Event::Time { now });
        Ok(())
    }
}

fn local_time_string() -> Result<String> {
    unsafe {
        let mut now = 0;
        time(&raw mut now);

        let mut tm: tm = core::mem::zeroed();

        if localtime_r(&raw const now, &raw mut tm).is_null() {
            bail!("failed to get time");
        }

        let fmt = c"%H:%M:%S | %b %d | %a";
        let mut buf = [0_i8; 64];

        let n = strftime(buf.as_mut_ptr(), buf.len(), fmt.as_ptr(), &raw const tm);

        if n == 0 {
            bail!("failed to format time");
        }

        Ok(core::ffi::CStr::from_ptr(buf.as_ptr())
            .to_string_lossy()
            .into_owned())
    }
}
