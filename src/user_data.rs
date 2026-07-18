use crate::sansio::Op;
use anyhow::{Result, ensure};

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum ModuleId {
    KbMod,
    NM,
    PW,
    Niri,
    Weather,
    Tray,
    Control,

    #[expect(clippy::upper_case_acronyms)]
    CPU,
    Memory,
    Timer,
}

const MAX: ModuleId = ModuleId::Timer;

impl ModuleId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Weather => "Weather",
            Self::KbMod => "KbMod",
            Self::NM => "NM",
            Self::PW => "PW",
            Self::Niri => "Niri",
            Self::Tray => "Tray",
            Self::Control => "Control",
            Self::CPU => "CPU",
            Self::Memory => "Memory",
            Self::Timer => "Timer",
        }
    }
}

impl From<ModuleId> for u8 {
    fn from(value: ModuleId) -> Self {
        value as Self
    }
}

impl TryFrom<u8> for ModuleId {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        ensure!(
            value <= MAX as u8,
            "received malformed ModuleId from io_uring: {value}"
        );
        Ok(unsafe { core::mem::transmute::<u8, Self>(value) })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(C, align(8))]
pub(crate) struct UserData {
    pub(crate) module_id: ModuleId,
    pub(crate) op: Op,
    pub(crate) req: u32,
}
const _: [u8; 8] = [0; size_of::<UserData>()];

static mut NEXT_REQUEST_ID: u32 = 1;
fn next_request_id() -> u32 {
    let request_id = unsafe { NEXT_REQUEST_ID };
    unsafe { NEXT_REQUEST_ID = NEXT_REQUEST_ID.wrapping_add(1) };
    request_id
}

impl UserData {
    pub(crate) fn new(module_id: ModuleId, op: Op) -> Self {
        Self {
            module_id,
            op,
            req: next_request_id(),
        }
    }
}

impl From<UserData> for u64 {
    fn from(user_data: UserData) -> Self {
        let mut bytes = [0_u8; 8];
        bytes[0] = user_data.module_id.into();
        bytes[1] = user_data.op as u8;
        bytes[2..6].copy_from_slice(&user_data.req.to_le_bytes());
        Self::from_le_bytes(bytes)
    }
}

impl TryFrom<u64> for UserData {
    type Error = anyhow::Error;

    fn try_from(value: u64) -> Result<Self> {
        let bytes: [u8; 8] = value.to_le_bytes();
        let module_id = ModuleId::try_from(bytes[0])?;
        let op = Op::try_from(bytes[1])?;
        let req = {
            let mut req = [0; 4];
            req.copy_from_slice(&bytes[2..6]);
            u32::from_le_bytes(req)
        };
        Ok(Self { module_id, op, req })
    }
}
