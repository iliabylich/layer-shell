use crate::{Event, liburing::IoUring, modules::hyprland::writer::CapsLock, user_data::UserData};
use anyhow::{Context as _, Result};
use reader::HyprlandReader;
use state::HyprlandState;
use std::collections::VecDeque;
use writer::{
    ActiveWorkspaceResource, DevicesResource, HyprlandWriter, WorkspaceListResource, WriterReply,
    WriterResource,
};

mod array_writer;
mod event;
mod reader;
mod state;
mod writer;

pub(crate) struct Hyprland {
    reader: Box<HyprlandReader>,
    writer: Box<HyprlandWriter>,
    state: HyprlandState,
    queue: VecDeque<Box<dyn WriterResource>>,
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
            queue: VecDeque::new(),
        }))
    }

    pub(crate) fn enqueue_get_caps_lock(&mut self) {
        self.queue.push_back(Box::new(CapsLock));
    }

    pub(crate) fn drain(&mut self, ring: &mut IoUring) -> Result<bool> {
        let mut drained = false;

        if self.writer.is_finished()
            && let Some(res) = self.queue.pop_front()
        {
            self.writer = writer(res)?;
        }

        loop {
            let mut drained_on_current_iteration = false;
            drained_on_current_iteration |= self.writer.drain(ring)?;
            drained_on_current_iteration |= self.reader.drain(ring)?;
            if !drained_on_current_iteration {
                break;
            }
            drained |= drained_on_current_iteration;
        }

        Ok(drained)
    }

    pub(crate) fn feed(
        &mut self,
        user_data: UserData,
        res: i32,
        events: &mut Vec<Event>,
    ) -> Result<()> {
        if let Some(reply) = self.writer.feed(user_data, res)? {
            match reply {
                WriterReply::WorkspaceList(workspace_ids) => {
                    self.state.init_workspace_ids(workspace_ids);
                    self.writer = writer(Box::new(ActiveWorkspaceResource))?;
                }
                WriterReply::ActiveWorkspace(id) => {
                    self.state.init_active_workspace(id);
                    self.writer = writer(Box::new(DevicesResource))?;
                }
                WriterReply::ActiveKeymap(active_keymap) => {
                    self.state.init_language(active_keymap);

                    for event in self.state.initial_events() {
                        events.push(event);
                    }
                }
                WriterReply::CapsLock(enabled) => {
                    events.push(Event::CapsLockToggled { enabled });
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
}

fn xdg_runtime_dir() -> Result<String> {
    std::env::var("XDG_RUNTIME_DIR").context("no XDG_RUNTIME_DIR variable")
}

fn hyprland_instance_signature() -> Result<String> {
    std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .context("no HYPRLAND_INSTANCE_SIGNATURE, are you in Hyprland?")
}
