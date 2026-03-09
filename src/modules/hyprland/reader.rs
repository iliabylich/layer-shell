use crate::{
    Event,
    modules::{
        Module,
        hyprland::state::{HyprlandDiff, HyprlandState},
    },
    sansio::{Satisfy, UnixSocketReader, Wants},
    user_data::ModuleId,
};
use anyhow::{Context as _, Result};
use libc::sockaddr_un;
use std::{cell::RefCell, rc::Rc};

pub(crate) struct HyprlandReader {
    socket_reader: UnixSocketReader,
    state: Rc<RefCell<HyprlandState>>,
}

impl Module for HyprlandReader {
    type Input = (sockaddr_un, Rc<RefCell<HyprlandState>>);
    type Output = ();
    type Error = anyhow::Error;

    const MODULE_ID: ModuleId = ModuleId::HyprlandReader;

    fn new((addr, state): Self::Input) -> Self {
        Self {
            socket_reader: UnixSocketReader::new(addr),
            state,
        }
    }

    fn wants(&mut self) -> Wants {
        self.socket_reader.wants()
    }

    fn satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
        events: &mut Vec<Event>,
    ) -> Result<Self::Output, Self::Error> {
        let Some((buf, len)) = self.socket_reader.satisfy(satisfy, res)? else {
            return Ok(());
        };

        let s = std::str::from_utf8(&buf[..len]).context("decoding error")?;
        let mut state = self.state.borrow_mut();

        for line in s.lines() {
            let Some(diff) = try_parse(line).context("parse error")? else {
                continue;
            };
            if let Some(event) = state.apply(diff) {
                events.push(event);
            }
        }
        Ok(())
    }

    fn tick(&mut self, _tick: u64) {}
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
        "activelayout" => HyprlandDiff::SetLanguage(last_substring()?.to_string()),
        _ => return Ok(None),
    };

    Ok(Some(event))
}
