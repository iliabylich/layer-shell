use core::sync::atomic::{AtomicU32, Ordering};

use crate::sansio::Op;

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
    Cpu,
    Memory,
    Timer,
}

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
            Self::Cpu => "Cpu",
            Self::Memory => "Memory",
            Self::Timer => "Timer",
        }
    }

    fn new(value: u8) -> Self {
        if value == Self::Weather as u8 {
            Self::Weather
        } else if value == Self::KbMod as u8 {
            Self::KbMod
        } else if value == Self::NM as u8 {
            Self::NM
        } else if value == Self::PW as u8 {
            Self::PW
        } else if value == Self::Niri as u8 {
            Self::Niri
        } else if value == Self::Tray as u8 {
            Self::Tray
        } else if value == Self::Control as u8 {
            Self::Control
        } else if value == Self::Cpu as u8 {
            Self::Cpu
        } else if value == Self::Memory as u8 {
            Self::Memory
        } else if value == Self::Timer as u8 {
            Self::Timer
        } else {
            unreachable!("can't build ModuleId from {value}")
        }
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

fn next_request_id() -> u32 {
    static NEXT_REQUEST_ID: AtomicU32 = AtomicU32::new(1);
    NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed)
}

impl UserData {
    pub(crate) fn new(module_id: ModuleId, op: Op) -> Self {
        Self {
            module_id,
            op,
            req: next_request_id(),
        }
    }

    pub(crate) fn encode(self) -> u64 {
        let mut bytes = [0_u8; 8];
        bytes[0] = self.module_id as u8;
        bytes[1] = self.op as u8;
        bytes[2..6].copy_from_slice(&self.req.to_le_bytes());
        u64::from_le_bytes(bytes)
    }

    pub(crate) fn decode(value: u64) -> Self {
        let bytes: [u8; 8] = value.to_le_bytes();
        let module_id = ModuleId::new(bytes[0]);
        let op = Op::new(bytes[1]);
        let req = {
            let mut req = [0; 4];
            req.copy_from_slice(&bytes[2..6]);
            u32::from_le_bytes(req)
        };
        Self { module_id, op, req }
    }
}
