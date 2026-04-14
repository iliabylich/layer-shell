use crate::{
    event_queue::EventQueue,
    modules::hyprland::state::{HyprlandDiff, HyprlandState},
    sansio::{Satisfy, UnixSocketReader, Wants},
    user_data::ModuleId,
    utils::StringRef,
};
use anyhow::{Context as _, Result};

pub(crate) struct HyprlandReader {
    socket_reader: UnixSocketReader,
}

impl HyprlandReader {
    pub(crate) fn new(socket_reader: UnixSocketReader) -> Self {
        Self { socket_reader }
    }

    pub(crate) const fn module_id(&self) -> ModuleId {
        ModuleId::HyprlandReader
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        self.socket_reader.wants()
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<()> {
        let Some((buf, len)) = self.socket_reader.satisfy(satisfy, res) else {
            return Ok(());
        };

        let s = core::str::from_utf8(&buf[..len]).context("decoding error")?;
        for line in s.lines() {
            let Some(diff) = try_parse(line).context("parse error")? else {
                continue;
            };
            if let Some(event) = HyprlandState::apply(diff) {
                EventQueue::push_back(event);
            }
        }
        Ok(())
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) {
        if let Err(err) = self.try_satisfy(satisfy, res) {
            log::error!("HyprlandReader has crashed: {satisfy:?} {res} {err:?}");
            self.socket_reader.stop();
        }
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
        "activelayout" => HyprlandDiff::SetLanguage(StringRef::new(last_substring()?)),
        _ => return Ok(None),
    };

    Ok(Some(event))
}
