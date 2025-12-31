use crate::{
    Event, UserData,
    liburing::{Actor, Cqe, IoUring, Pending},
};
use anyhow::{Context as _, Result};
use reader::HyprlandReader;
use state::HyprlandState;
use writer::{
    ActiveWorkspaceResource, DevicesResource, HyprlandWriter, WorkspaceListResource, WriterReply,
    WriterResource,
};

mod event;
mod reader;
mod state;
mod writer;

pub(crate) struct Hyprland {
    reader: HyprlandReader,
    writer: HyprlandWriter,
    state: HyprlandState,
}

fn reader() -> Result<HyprlandReader> {
    HyprlandReader::new(UserData::HyprlandReaderRead as u64)
}
fn writer(resource: Box<dyn WriterResource>) -> Result<HyprlandWriter> {
    HyprlandWriter::new(
        resource,
        UserData::HyprlandWriterSocket as u64,
        UserData::HyprlandWriterConnect as u64,
        UserData::HyprlandWriterWrite as u64,
        UserData::HyprlandWriterRead as u64,
        UserData::HyprlandWriterClose as u64,
    )
}

impl Hyprland {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self {
            reader: reader()?,
            writer: writer(Box::new(WorkspaceListResource))?,
            state: HyprlandState::default(),
        })
    }
}

impl Actor for Hyprland {
    fn drain_once(
        &mut self,
        ring: &mut IoUring,
        pending: &mut Pending,
        _events: &mut Vec<Event>,
    ) -> Result<bool> {
        let mut drained = false;

        drained |= self.writer.drain(ring)?;
        drained |= self.reader.drain(ring, pending)?;

        Ok(drained)
    }

    fn feed(&mut self, _ring: &mut IoUring, cqe: Cqe, events: &mut Vec<Event>) -> Result<()> {
        if let Some(reply) = self.writer.feed(cqe)? {
            match reply {
                WriterReply::WorkspaceList(workspaces) => {
                    let workspace_ids = workspaces.into_iter().map(|w| w.id).collect();
                    self.state.init_workspace_ids(workspace_ids);
                    self.writer = writer(Box::new(ActiveWorkspaceResource))?;
                }
                WriterReply::ActiveWorkspace(workspace) => {
                    self.state.init_active_workspace(workspace.id);
                    self.writer = writer(Box::new(DevicesResource))?;
                }
                WriterReply::Devices(devices) => {
                    let main_keyboard = devices
                        .keyboards
                        .into_iter()
                        .find(|keyboard| keyboard.main)
                        .context("expected at least one hyprland device")?;
                    self.state.init_language(main_keyboard.active_keymap);

                    for event in self.state.initial_events() {
                        events.push(event);
                    }
                }
            }
        }

        let mut hevents = vec![];
        self.reader.feed(cqe, &mut hevents)?;
        for hevent in hevents {
            let event = self.state.apply(hevent);
            events.push(event);
        }

        Ok(())
    }
}

pub(crate) fn xdg_runtime_dir() -> Result<String> {
    std::env::var("XDG_RUNTIME_DIR").context("no XDG_RUNTIME_DIR variable")
}

pub(crate) fn hyprland_instance_signature() -> Result<String> {
    std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .context("no HYPRLAND_INSTANCE_SIGNATURE, are you in Hyprland?")
}
