use crate::{
    Event,
    event_queue::EventQueue,
    utils::{StringRef, StringRefExt},
};
use alloc::string::String;
use anyhow::{Result, bail};

pub(crate) struct Clock;

impl Clock {
    pub(crate) fn tick(events: &mut EventQueue) -> Result<()> {
        let now = StringRef::new(&local_time_string()?);
        events.push_back(Event::Time { now });
        Ok(())
    }
}

fn local_time_string() -> Result<String> {
    unsafe {
        let mut now = 0;
        libc::time(&raw mut now);

        let mut tm: libc::tm = core::mem::zeroed();

        if libc::localtime_r(&raw const now, &raw mut tm).is_null() {
            bail!("failed to get time");
        }

        let fmt = c"%H:%M:%S | %b %d | %a";
        let mut buf = [0_i8; 64];

        let n = libc::strftime(buf.as_mut_ptr(), buf.len(), fmt.as_ptr(), &raw const tm);

        if n == 0 {
            bail!("failed to format time");
        }

        Ok(core::ffi::CStr::from_ptr(buf.as_ptr())
            .to_string_lossy()
            .into_owned())
    }
}
