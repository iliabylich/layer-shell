use crate::{
    UserData,
    liburing::IoUring,
    modules::hyprland::state::HyprlandDiff,
    sansio::{Satisfy, UnixSocketReader, Wants},
    unix_socket::new_unix_socket,
    user_data::ModuleId,
};
use anyhow::{Context as _, Result};

pub(crate) struct HyprlandReader {
    socket_reader: UnixSocketReader,
}

impl HyprlandReader {
    pub(crate) const MODULE_ID: ModuleId = ModuleId::HyprlandReader;

    pub(crate) fn new(xdg_runtime_dir: &str, hyprland_instance_signature: &str) -> Box<Self> {
        let addr = new_unix_socket(
            format!("{xdg_runtime_dir}/hypr/{hyprland_instance_signature}/.socket2.sock")
                .as_bytes(),
        );

        Box::new(Self {
            socket_reader: UnixSocketReader::new(addr),
        })
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
            Wants::Read { fd, buf } => {
                sqe.prep_read(fd, buf.as_mut_ptr(), buf.len());
                sqe.set_user_data(UserData::new(Self::MODULE_ID, Satisfy::Read));
            }
            other => unreachable!("HyprlandReader never wants {other:?}"),
        }
    }

    pub(crate) fn init(&mut self) {
        self.schedule_wanted_operation();
    }

    pub(crate) fn process(&mut self, op: u8, res: i32) -> Result<Vec<HyprlandDiff>> {
        let satisfy = Satisfy::from(op);
        let mut out = vec![];

        match self.socket_reader.satisfy(satisfy, res)? {
            Some((buf, len)) => {
                let s = std::str::from_utf8(&buf[..len]).context("decoding error")?;
                for line in s.lines() {
                    // HyprlandDiff
                    let Some(diff) = try_parse(line).context("parse error")? else {
                        continue;
                    };
                    out.push(diff);
                }
            }
            None => {}
        }

        self.schedule_wanted_operation();
        Ok(out)
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
