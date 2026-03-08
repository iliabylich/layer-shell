use crate::{
    Event, UserData,
    liburing::IoUring,
    modules::hyprland::state::{HyprlandDiff, HyprlandState},
    sansio::{Satisfy, UnixSocketReader, Wants},
    unix_socket::new_unix_socket,
    user_data::ModuleId,
};
use anyhow::{Context as _, Result};
use std::{cell::RefCell, rc::Rc};

pub(crate) struct HyprlandReader {
    socket_reader: UnixSocketReader,
    state: Rc<RefCell<HyprlandState>>,
}

impl HyprlandReader {
    pub(crate) const MODULE_ID: ModuleId = ModuleId::HyprlandReader;

    pub(crate) fn new(
        xdg_runtime_dir: &str,
        hyprland_instance_signature: &str,
        state: Rc<RefCell<HyprlandState>>,
    ) -> Self {
        let addr = new_unix_socket(
            format!("{xdg_runtime_dir}/hypr/{hyprland_instance_signature}/.socket2.sock")
                .as_bytes(),
        );

        Self {
            socket_reader: UnixSocketReader::new(addr),
            state,
        }
    }

    fn schedule_wanted_operation(&mut self) {
        let mut sqe = IoUring::get_sqe();

        match self.socket_reader.wants() {
            Wants::Socket { domain, r#type } => {
                sqe.prep_socket(domain, r#type, 0, 0);
                sqe.set_user_data(UserData::new(Self::MODULE_ID, Satisfy::Socket));
            }
            Wants::Connect { fd, addr, addrlen } => {
                sqe.prep_connect(fd, addr, addrlen);
                sqe.set_user_data(UserData::new(Self::MODULE_ID, Satisfy::Connect));
            }
            Wants::Read { fd, buf, len } => {
                sqe.prep_read(fd, buf, len);
                sqe.set_user_data(UserData::new(Self::MODULE_ID, Satisfy::Read));
            }
            other => unreachable!("HyprlandReader never wants {other:?}"),
        }
    }

    pub(crate) fn init(&mut self) {
        self.schedule_wanted_operation();
    }

    fn try_process(&mut self, op: u8, res: i32, events: &mut Vec<Event>) -> Result<()> {
        let satisfy = Satisfy::from(op);

        match self.socket_reader.satisfy(satisfy, res)? {
            Some((buf, len)) => {
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
            }
            None => {}
        }

        self.schedule_wanted_operation();
        Ok(())
    }

    pub(crate) fn process(&mut self, op: u8, res: i32, events: &mut Vec<Event>) {
        if let Err(err) = self.try_process(op, res, events) {
            log::error!("HyprlandReader: {err:?}")
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
        "activelayout" => HyprlandDiff::SetLanguage(last_substring()?.to_string()),
        _ => return Ok(None),
    };

    Ok(Some(event))
}
