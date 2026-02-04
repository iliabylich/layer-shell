use std::cell::Cell;

use anyhow::{Result, ensure};

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum ModuleId {
    Weather = 1,
    GeoLocation,
    HyprlandReader,
    HyprlandWriter,

    SessionDBusAuth,
    SessionDBusReader,
    SessionDBusWriter,

    SystemDBusAuth,
    SystemDBusReader,
    SystemDBusWriter,

    #[expect(clippy::upper_case_acronyms)]
    CPU,
    Memory,
    TimerFD,
    Max,
}

impl From<ModuleId> for u8 {
    fn from(value: ModuleId) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for ModuleId {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        ensure!(value < Self::Max as u8);
        unsafe { Ok(std::mem::transmute::<u8, Self>(value)) }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(C, align(8))]
pub(crate) struct UserData {
    pub(crate) module_id: ModuleId,
    pub(crate) op: u8,
    pub(crate) req: u32,
}
const _: [u8; 8] = [0; std::mem::size_of::<UserData>()];

thread_local! {
    static NEXT_REQUEST_ID: Cell<u32> = const { Cell::new(1) };
}
fn next_request_id() -> u32 {
    let request_id = NEXT_REQUEST_ID.get();
    NEXT_REQUEST_ID.set(request_id + 1);
    request_id
}

impl UserData {
    pub(crate) fn new(module_id: ModuleId, op: impl Into<u8>) -> Self {
        Self {
            module_id,
            op: op.into(),
            req: next_request_id(),
        }
    }
}

impl From<UserData> for u64 {
    fn from(user_data: UserData) -> Self {
        let mut bytes = [0_u8; 8];
        bytes[0] = user_data.module_id.into();
        bytes[1] = user_data.op;
        bytes[2..6].copy_from_slice(&user_data.req.to_le_bytes());
        u64::from_le_bytes(bytes)
    }
}

impl TryFrom<u64> for UserData {
    type Error = anyhow::Error;

    fn try_from(value: u64) -> Result<Self> {
        let bytes: [u8; 8] = value.to_le_bytes();
        let module_id = ModuleId::try_from(bytes[0])?;
        let op = bytes[1];
        let req = {
            let mut req = [0; 4];
            req.copy_from_slice(&bytes[2..6]);
            u32::from_le_bytes(req)
        };
        Ok(Self { module_id, op, req })
    }
}
