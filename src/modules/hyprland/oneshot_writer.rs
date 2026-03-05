use crate::{
    UserData,
    liburing::IoUring,
    modules::hyprland::{resources::WriterResource, state::HyprlandDiff},
    sansio::{Satisfy, UnixSocketOneshotWriter, Wants},
    user_data::ModuleId,
};
use anyhow::{Context, Result};
use libc::sockaddr_un;

pub(crate) struct OneshotWriter {
    socket_writer: UnixSocketOneshotWriter,
    resource: Box<dyn WriterResource>,
}

impl OneshotWriter {
    pub(crate) const MODULE_ID: ModuleId = ModuleId::HyprlandWriter;

    pub(crate) fn new(addr: sockaddr_un, resource: Box<dyn WriterResource>) -> Self {
        Self {
            socket_writer: UnixSocketOneshotWriter::new(addr, resource.command().as_ref()),
            resource,
        }
    }

    fn schedule_wanted_operation(&mut self) {
        let mut sqe = IoUring::get_sqe();

        match self.socket_writer.wants() {
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
            Wants::Write { fd, buf } => {
                sqe.prep_write(fd, buf.as_ptr(), buf.len());
                sqe.set_user_data(UserData::new(Self::MODULE_ID, Satisfy::Write));
            }
            Wants::Close { fd } => {
                sqe.prep_close(fd);
                sqe.set_user_data(UserData::new(Self::MODULE_ID, Satisfy::Close));
            }
            Wants::Nothing => unreachable!(),
        }
    }

    pub(crate) fn init(&mut self) {
        self.schedule_wanted_operation();
    }

    pub(crate) fn process(&mut self, op: u8, res: i32) -> Result<Option<HyprlandDiff>> {
        let satisfy = Satisfy::from(op);

        let Some(buf) = self
            .socket_writer
            .satisfy(satisfy, res)
            .context("Hyprland::Writer")?
        else {
            self.schedule_wanted_operation();
            return Ok(None);
        };

        let json = std::str::from_utf8(buf).context("decoding error")?;
        self.resource.parse(json).context("parse error")
    }
}
