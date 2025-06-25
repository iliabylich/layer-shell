use ffi::CArray;

#[derive(Debug)]
#[repr(C)]
pub struct CpuUsageEvent {
    pub usage_per_core: CArray<u8>,
}
