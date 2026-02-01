use crate::{
    UserData,
    liburing::IoUring,
    modules::hyprland::{array_writer::ArrayWriter, hyprland_instance_signature, xdg_runtime_dir},
    user_data::ModuleId,
};
use anyhow::{Context as _, Result, ensure};
use core::fmt::Write;
use libc::{AF_UNIX, SOCK_STREAM, sockaddr, sockaddr_un};
use serde::Deserialize;
use std::{borrow::Cow, collections::HashSet};

pub(crate) trait WriterResource {
    fn command(&self) -> Cow<'static, str>;
    fn parse(&self, json: &str) -> Result<WriterReply>;
}

pub(crate) struct WorkspaceListResource;

impl WriterResource for WorkspaceListResource {
    fn command(&self) -> Cow<'static, str> {
        Cow::Borrowed("[[BATCH]]j/workspaces")
    }

    fn parse(&self, json: &str) -> Result<WriterReply> {
        #[derive(Debug, Deserialize)]
        struct Workspace {
            id: u64,
        }
        let workspaces: Vec<Workspace> =
            serde_json::from_str(json).context("malformed workspaces response")?;

        let workspace_ids = workspaces.into_iter().map(|w| w.id).collect();

        Ok(WriterReply::WorkspaceList(workspace_ids))
    }
}

pub(crate) struct ActiveWorkspaceResource;
impl WriterResource for ActiveWorkspaceResource {
    fn command(&self) -> Cow<'static, str> {
        Cow::Borrowed("[[BATCH]]j/activeworkspace")
    }

    fn parse(&self, json: &str) -> Result<WriterReply> {
        #[derive(Deserialize)]
        struct Workspace {
            id: u64,
        }
        let workspace: Workspace =
            serde_json::from_str(json).context("malformed activeworkspace response")?;
        Ok(WriterReply::ActiveWorkspace(workspace.id))
    }
}

pub(crate) struct DevicesResource;
impl WriterResource for DevicesResource {
    fn command(&self) -> Cow<'static, str> {
        Cow::Borrowed("[[BATCH]]j/devices")
    }

    fn parse(&self, json: &str) -> Result<WriterReply> {
        #[derive(Deserialize)]
        struct Devices {
            keyboards: Vec<Keyboard>,
        }
        #[derive(Deserialize)]
        struct Keyboard {
            main: bool,
            active_keymap: String,
        }

        let devices: Devices = serde_json::from_str(json).context("malformed devices response")?;

        let active_keymap = devices
            .keyboards
            .into_iter()
            .find(|keyboard| keyboard.main)
            .context("expected at least one hyprland device")?
            .active_keymap;

        Ok(WriterReply::ActiveKeymap(active_keymap))
    }
}

pub(crate) struct CapsLock;
impl WriterResource for CapsLock {
    fn command(&self) -> Cow<'static, str> {
        Cow::Borrowed("[[BATCH]]j/devices")
    }

    fn parse(&self, json: &str) -> Result<WriterReply> {
        #[derive(Deserialize)]
        struct Devices {
            keyboards: Vec<Keyboard>,
        }
        #[derive(Deserialize)]
        struct Keyboard {
            main: bool,
            #[serde(rename = "capsLock")]
            caps_lock: bool,
        }

        let devices: Devices = serde_json::from_str(json).context("malformed devices response")?;
        let main_keyboard = devices
            .keyboards
            .into_iter()
            .find(|keyboard| keyboard.main)
            .context("expected at least one hyprland device")?;

        Ok(WriterReply::CapsLock(main_keyboard.caps_lock))
    }
}

pub(crate) struct Dispatch {
    cmd: String,
}
impl Dispatch {
    pub(crate) fn new(cmd: String) -> Self {
        Self { cmd }
    }
}
impl WriterResource for Dispatch {
    fn command(&self) -> Cow<'static, str> {
        Cow::Owned(format!("dispatch {}", self.cmd))
    }

    fn parse(&self, reply: &str) -> Result<WriterReply> {
        ensure!(
            reply == "ok",
            "invalid response from hyprctl dispatch: expected 'ok', got {reply:?}",
        );
        Ok(WriterReply::None)
    }
}

#[derive(Debug, Clone, Copy)]
enum State {
    Initial,
    SocketRequested,
    SocketAcquired,
    Connecting,
    Connected,
    Writing,
    Written,
    Reading,
    Read,
    Closing,
    Closed,
}

#[derive(Debug)]
pub(crate) enum WriterReply {
    WorkspaceList(HashSet<u64>),
    ActiveWorkspace(u64),
    ActiveKeymap(String),
    CapsLock(bool),
    None,
}

#[repr(u8)]
enum Op {
    Socket,
    Connect,
    Write,
    Read,
    Close,
}

pub(crate) struct HyprlandWriter {
    fd: i32,
    addr: sockaddr_un,
    buf: [u8; 4_096],
    resource: Box<dyn WriterResource>,
    state: State,
    reply: Option<WriterReply>,
}

impl HyprlandWriter {
    pub(crate) fn new(resource: Box<dyn WriterResource>) -> Result<Box<Self>> {
        let addr = sockaddr_un {
            sun_family: AF_UNIX as u16,
            sun_path: {
                let path = format!(
                    "{}/hypr/{}/.socket.sock",
                    xdg_runtime_dir()?,
                    hyprland_instance_signature()?
                );
                let path = unsafe { std::mem::transmute::<&[u8], &[i8]>(path.as_bytes()) };
                let mut out = [0; 108];
                out[..path.len()].copy_from_slice(path);
                out
            },
        };

        Ok(Box::new(Self {
            fd: -1,
            addr,
            buf: [0; 4_096],
            resource,
            state: State::Initial,
            reply: None,
        }))
    }

    pub(crate) fn drain(&mut self, ring: &mut IoUring) -> Result<bool> {
        match self.state {
            State::Initial => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_socket(AF_UNIX, SOCK_STREAM, 0, 0);
                sqe.set_user_data(UserData::new(ModuleId::HyprlandWriter, Op::Socket as u8));
                self.state = State::SocketRequested;
                Ok(true)
            }
            State::SocketRequested => Ok(false),

            State::SocketAcquired => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_connect(
                    self.fd,
                    (&self.addr as *const sockaddr_un).cast::<sockaddr>(),
                    std::mem::size_of::<sockaddr_un>() as u32,
                );
                sqe.set_user_data(UserData::new(ModuleId::HyprlandWriter, Op::Connect as u8));
                self.state = State::Connecting;
                Ok(true)
            }
            State::Connecting => Ok(false),

            State::Connected => {
                let mut writer = ArrayWriter::new(&mut self.buf);
                write!(&mut writer, "{}", self.resource.command())?;
                let buflen = writer.offset();

                let mut sqe = ring.get_sqe()?;
                sqe.prep_write(self.fd, self.buf.as_ptr(), buflen);
                sqe.set_user_data(UserData::new(ModuleId::HyprlandWriter, Op::Write as u8));
                self.state = State::Writing;
                Ok(true)
            }
            State::Writing => Ok(false),

            State::Written => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
                sqe.set_user_data(UserData::new(ModuleId::HyprlandWriter, Op::Read as u8));
                self.state = State::Reading;
                Ok(true)
            }
            State::Reading => Ok(false),

            State::Read => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_close(self.fd);
                sqe.set_user_data(UserData::new(ModuleId::HyprlandWriter, Op::Close as u8));
                self.state = State::Closing;
                Ok(true)
            }
            State::Closing => Ok(false),

            State::Closed => Ok(false),
        }
    }

    pub(crate) fn feed(&mut self, op_id: u8, res: i32) -> Result<Option<WriterReply>> {
        if op_id == Op::Socket as u8 {
            ensure!(
                matches!(self.state, State::SocketRequested),
                "malformed state, expected SocketRequested, got {:?}",
                self.state
            );

            let fd = res;
            ensure!(fd > 0);
            self.fd = fd;
            self.state = State::SocketAcquired;
            return Ok(None);
        }

        if op_id == Op::Connect as u8 {
            ensure!(
                matches!(self.state, State::Connecting),
                "malformed state, expected Connecting, got {:?}",
                self.state
            );

            ensure!(res >= 0);
            self.state = State::Connected;
            return Ok(None);
        }

        if op_id == Op::Write as u8 {
            ensure!(
                matches!(self.state, State::Writing),
                "malformed state, expected Writing, got {:?}",
                self.state
            );

            ensure!(res > 0);
            self.state = State::Written;
            return Ok(None);
        }

        if op_id == Op::Read as u8 {
            ensure!(
                matches!(self.state, State::Reading),
                "malformed state, expected Reading, got {:?}",
                self.state
            );

            ensure!(res > 0);
            let len = res as usize;
            let json = std::str::from_utf8(&self.buf[..len])?;
            self.reply = Some(self.resource.parse(json)?);
            self.state = State::Read;

            return Ok(None);
        }

        if op_id == Op::Close as u8 {
            ensure!(
                matches!(self.state, State::Closing),
                "malformed state, expected Closing, got {:?}",
                self.state
            );

            ensure!(res >= 0);
            self.state = State::Closed;
            return Ok(self.reply.take());
        }

        Ok(None)
    }

    pub(crate) fn is_finished(&self) -> bool {
        matches!(self.state, State::Closed)
    }
}
