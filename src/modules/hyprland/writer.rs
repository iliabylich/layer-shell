use crate::{
    UserData,
    liburing::IoUring,
    macros::{define_op, report_and_exit},
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
        if reply != "ok" {
            log::error!("invalid response from hyprctl dispatch: expected 'ok', got {reply:?}");
        }
        Ok(WriterReply::None)
    }
}

#[derive(Debug)]
pub(crate) enum WriterReply {
    WorkspaceList(HashSet<u64>),
    ActiveWorkspace(u64),
    ActiveKeymap(String),
    CapsLock(bool),
    None,
}

define_op!("Hyprland Writer", Socket, Connect, Write, Read, Close);

pub(crate) struct HyprlandWriter {
    fd: i32,
    addr: sockaddr_un,
    buf: [u8; 4_096],
    resource: Box<dyn WriterResource>,
    reply: Option<WriterReply>,
    healthy: bool,
}

impl HyprlandWriter {
    pub(crate) fn new(resource: Box<dyn WriterResource>) -> Box<Self> {
        Box::new(Self {
            fd: -1,
            addr: unsafe { std::mem::zeroed() },
            buf: [0; 4_096],
            resource,
            reply: None,
            healthy: true,
        })
    }

    fn schedule_socket(&self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_socket(AF_UNIX, SOCK_STREAM, 0, 0);
        sqe.set_user_data(UserData::new(ModuleId::HyprlandWriter, Op::Socket));
    }

    fn schedule_connect(&mut self) {
        let Some(xdg_runtime_dir) = xdg_runtime_dir() else {
            return;
        };
        let Some(hyprland_instance_signature) = hyprland_instance_signature() else {
            return;
        };

        self.addr = sockaddr_un {
            sun_family: AF_UNIX as u16,
            sun_path: {
                let path =
                    format!("{xdg_runtime_dir}/hypr/{hyprland_instance_signature}/.socket.sock");
                let path = unsafe { std::mem::transmute::<&[u8], &[i8]>(path.as_bytes()) };
                let mut out = [0; 108];
                out[..path.len()].copy_from_slice(path);
                out
            },
        };

        let mut sqe = IoUring::get_sqe();
        sqe.prep_connect(
            self.fd,
            (&self.addr as *const sockaddr_un).cast::<sockaddr>(),
            std::mem::size_of::<sockaddr_un>() as u32,
        );
        sqe.set_user_data(UserData::new(ModuleId::HyprlandWriter, Op::Connect));
    }

    fn schedule_write(&mut self) {
        let mut writer = ArrayWriter::new(&mut self.buf);
        write!(&mut writer, "{}", self.resource.command())
            .unwrap_or_else(|err| report_and_exit!("failed to write command to buffer: {err:?}"));
        let buflen = writer.offset();

        let mut sqe = IoUring::get_sqe();
        sqe.prep_write(self.fd, self.buf.as_ptr(), buflen);
        sqe.set_user_data(UserData::new(ModuleId::HyprlandWriter, Op::Write));
    }

    fn schedule_read(&mut self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
        sqe.set_user_data(UserData::new(ModuleId::HyprlandWriter, Op::Read));
    }

    fn schedule_close(&self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_close(self.fd);
        sqe.set_user_data(UserData::new(ModuleId::HyprlandWriter, Op::Close));
    }

    pub(crate) fn init(&mut self) {
        self.schedule_socket()
    }

    fn try_process(&mut self, op: Op, res: i32) -> Result<Option<WriterReply>> {
        match op {
            Op::Socket => {
                ensure!(res > 0);
                self.fd = res;
                self.schedule_connect();
                Ok(None)
            }
            Op::Connect => {
                ensure!(res >= 0);
                self.schedule_write();
                Ok(None)
            }
            Op::Write => {
                ensure!(res >= 0);
                self.schedule_read();
                Ok(None)
            }
            Op::Read => {
                ensure!(res > 0);
                let len = res as usize;

                let json = std::str::from_utf8(&self.buf[..len]).context("decoding error")?;
                let reply = self.resource.parse(json).context("parse error")?;

                self.reply = Some(reply);
                self.schedule_close();
                Ok(None)
            }
            Op::Close => {
                ensure!(res >= 0, "res is {res}");
                Ok(self.reply.take())
            }
        }
    }

    pub(crate) fn process(&mut self, op: u8, res: i32) -> Option<WriterReply> {
        if !self.healthy {
            return None;
        }

        let op = Op::from(op);

        match self.try_process(op, res) {
            Ok(ok) => ok,
            Err(err) => {
                log::error!("Hyprland::Writer::{op:?}({res} {err:?}");
                self.healthy = false;
                None
            }
        }
    }
}
