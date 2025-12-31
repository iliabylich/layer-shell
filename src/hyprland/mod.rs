use crate::{
    Event,
    liburing::{Actor, IoUring},
    user_data::UserData,
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
    reader: Box<HyprlandReader>,
    writer: Box<HyprlandWriter>,
    state: HyprlandState,
}

fn reader() -> Result<Box<HyprlandReader>> {
    HyprlandReader::new()
}
fn writer(resource: Box<dyn WriterResource>) -> Result<Box<HyprlandWriter>> {
    HyprlandWriter::new(resource)
}

impl Hyprland {
    pub(crate) fn new() -> Result<Box<Self>> {
        Ok(Box::new(Self {
            reader: reader()?,
            writer: writer(Box::new(WorkspaceListResource))?,
            state: HyprlandState::default(),
        }))
    }
}

impl Actor for Hyprland {
    fn drain_once(&mut self, ring: &mut IoUring, _events: &mut Vec<Event>) -> Result<bool> {
        let mut drained = false;

        drained |= self.writer.drain(ring)?;
        drained |= self.reader.drain(ring)?;

        Ok(drained)
    }

    fn feed(
        &mut self,
        _ring: &mut IoUring,
        user_data: UserData,
        res: i32,
        events: &mut Vec<Event>,
    ) -> Result<()> {
        if let Some(reply) = self.writer.feed(user_data, res)? {
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
        self.reader.feed(user_data, res, &mut hevents)?;
        for hevent in hevents {
            let event = self.state.apply(hevent);
            events.push(event);
        }

        Ok(())
    }

    fn on_tick(&mut self, _tick: crate::timerfd::Tick) -> Result<()> {
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
