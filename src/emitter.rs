use crate::IoEvent;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Emitter {
    callback: extern "C" fn(event: &IoEvent, *mut core::ffi::c_void),
    data: *mut core::ffi::c_void,
}

impl Emitter {
    pub(crate) const fn new(
        callback: extern "C" fn(event: &IoEvent, *mut core::ffi::c_void),
        data: *mut core::ffi::c_void,
    ) -> Self {
        Self { callback, data }
    }

    pub(crate) fn emit(&self, event: &IoEvent) {
        log::info!(target: "IO", "{event:?}");
        (self.callback)(event, self.data);
    }
}
