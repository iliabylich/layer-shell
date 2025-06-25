use ffi::CString;

#[derive(Debug)]
#[repr(C)]
pub struct ClockEvent {
    pub time: CString,
}
