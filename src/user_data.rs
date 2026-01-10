use std::cell::Cell;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum UserData {
    GetLocationSocket = 1,
    GetLocationConnect,
    GetLocationRead,
    GetLocationWrite,
    GetLocationClose,

    GetWeatherSocket,
    GetWeatherConnect,
    GetWeatherRead,
    GetWeatherWrite,
    GetWeatherClose,

    HyprlandReaderRead,

    HyprlandWriterSocket,
    HyprlandWriterConnect,
    HyprlandWriterWrite,
    HyprlandWriterRead,
    HyprlandWriterClose,

    SessionDBusAuthWriteZero,
    SessionDBusAuthWriteAuthExternal,
    SessionDBusAuthReadData,
    SessionDBusAuthWriteData,
    SessionDBusAuthReadGUID,
    SessionDBusAuthWriteBegin,

    SessionDBusReadHeader,
    SessionDBusReadBody,
    SessionDBusWrite,

    SystemDBusAuthWriteZero,
    SystemDBusAuthWriteAuthExternal,
    SystemDBusAuthReadData,
    SystemDBusAuthWriteData,
    SystemDBusAuthReadGUID,
    SystemDBusAuthWriteBegin,

    SystemDBusReadHeader,
    SystemDBusReadBody,
    SystemDBusWrite,

    CpuRead,

    MemoryRead,

    TimerfdRead,

    Max,
}

thread_local! {
    static NEXT_REQUEST_ID: Cell<u32> = const { Cell::new(1) };
}
fn next_request_id() -> u32 {
    let request_id = NEXT_REQUEST_ID.get();
    NEXT_REQUEST_ID.set(request_id + 1);
    request_id
}

impl UserData {
    pub(crate) fn as_u64(self) -> u64 {
        let request_id = next_request_id();
        eprintln!("==> {self:?}({request_id})");
        let mut bytes = [0; 8];
        bytes[..4].copy_from_slice(&request_id.to_le_bytes());
        bytes[7] = self as u8;
        u64::from_le_bytes(bytes)
    }

    pub(crate) fn from_u64(n: u64, res: i32) -> (Self, u32) {
        let bytes: [u8; 8] = n.to_le_bytes();
        let mut high = [0; 4];
        high.copy_from_slice(&bytes[..4]);
        let request_id = { u32::from_le_bytes(high) };
        assert!(bytes[7] < Self::Max as u8);
        let this: Self = unsafe { std::mem::transmute(bytes[7]) };
        eprintln!("<== {this:?}({request_id}, {res})");
        (this, request_id)
    }
}
