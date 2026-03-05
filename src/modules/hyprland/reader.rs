use crate::{
    UserData,
    liburing::IoUring,
    modules::hyprland::event::HyprlandEvent,
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

    pub(crate) fn process(
        &mut self,
        op: u8,
        res: i32,
        events: &mut Vec<HyprlandEvent>,
    ) -> Result<()> {
        let satisfy = Satisfy::from(op);

        match self.socket_reader.satisfy(satisfy, res)? {
            Some((buf, len)) => {
                let s = std::str::from_utf8(&buf[..len]).context("decoding error")?;
                for line in s.lines() {
                    if let Some(event) = HyprlandEvent::try_parse(line).context("parse error")? {
                        events.push(event)
                    };
                }
            }
            None => {}
        }

        self.schedule_wanted_operation();
        Ok(())
    }
}
