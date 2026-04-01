use crate::{
    event_queue::EventQueue,
    ffi::ShortString,
    modules::hyprland::state::{HyprlandDiff, HyprlandState},
    sansio::{Satisfy, UnixSocketReader, Wants},
    user_data::ModuleId,
};
use anyhow::{Context as _, Result};
use libc::sockaddr_un;

pub(crate) struct HyprlandReader {
    socket_reader: UnixSocketReader,
    state: HyprlandState,
}

impl HyprlandReader {
    pub(crate) fn new(addr: sockaddr_un, state: HyprlandState) -> Self {
        Self {
            socket_reader: UnixSocketReader::new(addr),
            state,
        }
    }

    pub(crate) const fn module_id(&self) -> ModuleId {
        ModuleId::HyprlandReader
    }

    pub(crate) fn wants(&mut self) -> Wants {
        self.socket_reader.wants()
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<()> {
        let Some((buf, len)) = self.socket_reader.satisfy(satisfy, res)? else {
            return Ok(());
        };

        let s = core::str::from_utf8(&buf[..len]).context("decoding error")?;
        for line in s.lines() {
            let Some(diff) = try_parse(line).context("parse error")? else {
                continue;
            };
            if let Some(event) = self.state.apply(diff) {
                EventQueue::push_back(event);
            }
        }
        Ok(())
    }
}

fn try_parse(line: &str) -> Result<Option<HyprlandDiff>> {
    let (event, payload) = line.split_once(">>").with_context(|| {
        format!("malformed line from Hyprland reader socket: {line:?} (expected >> separator)")
    })?;

    let num_payload = || {
        payload
            .parse::<u64>()
            .with_context(|| format!("non-numeric payload of {event} event: {payload:?}"))
    };

    let last_substring = || {
        payload.split(",").last().with_context(|| {
            format!("expected comma separator in the payload of {event}, got {payload:?}")
        })
    };

    let event = match event {
        "createworkspace" => HyprlandDiff::AddWorkspaceId(num_payload()?),
        "destroyworkspace" => HyprlandDiff::RemoveWorkspaceId(num_payload()?),
        "workspace" => HyprlandDiff::SetActiveWorkspaceId(num_payload()?),
        "activelayout" => HyprlandDiff::SetLanguage(ShortString::from(last_substring()?)),
        _ => return Ok(None),
    };

    Ok(Some(event))
}
