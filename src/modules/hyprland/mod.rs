use crate::{Event, liburing::IoUring, modules::hyprland::writer::CapsLock};
use anyhow::{Context as _, Result};
use reader::HyprlandReader;
use state::HyprlandState;
use std::collections::VecDeque;
use writer::{
    ActiveWorkspaceResource, DevicesResource, Dispatch, HyprlandWriter, WorkspaceListResource,
    WriterReply, WriterResource,
};

mod array_writer;
mod event;
mod reader;
mod state;
mod writer;

pub(crate) struct Hyprland {
    reader: Box<HyprlandReader>,
    writer: Option<Box<HyprlandWriter>>,
    state: HyprlandState,
    queue: VecDeque<Box<dyn WriterResource>>,
}

fn new_reader() -> Result<Box<HyprlandReader>> {
    HyprlandReader::new()
}
fn new_writer(resource: Box<dyn WriterResource>) -> Result<Box<HyprlandWriter>> {
    HyprlandWriter::new(resource)
}

impl Hyprland {
    pub(crate) fn new() -> Result<Box<Self>> {
        Ok(Box::new(Self {
            reader: new_reader()?,
            writer: Some(new_writer(Box::new(WorkspaceListResource))?),
            state: HyprlandState::default(),
            queue: VecDeque::from([
                Box::new(ActiveWorkspaceResource) as Box<dyn WriterResource>,
                Box::new(DevicesResource),
            ]),
        }))
    }

    pub(crate) fn init(&mut self, ring: &mut IoUring) -> Result<()> {
        self.reader.init(ring)?;
        if let Some(writer) = self.writer.as_mut() {
            writer.init(ring)?;
        }
        Ok(())
    }

    pub(crate) fn process_reader(
        &mut self,
        op: u8,
        res: i32,
        ring: &mut IoUring,
        events: &mut Vec<Event>,
    ) -> Result<()> {
        let mut hevents = vec![];
        self.reader.process(op, res, ring, &mut hevents)?;
        for hevent in hevents {
            let event = self.state.apply(hevent);
            events.push(event);
        }

        Ok(())
    }

    pub(crate) fn process_writer(
        &mut self,
        op: u8,
        res: i32,
        ring: &mut IoUring,
        events: &mut Vec<Event>,
    ) -> Result<()> {
        if let Some(writer) = self.writer.as_mut()
            && let Some(reply) = writer.process(op, res, ring)?
        {
            match reply {
                WriterReply::WorkspaceList(workspace_ids) => {
                    self.state.init_workspace_ids(workspace_ids);
                }
                WriterReply::ActiveWorkspace(id) => {
                    self.state.init_active_workspace(id);
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
                WriterReply::None => {}
            }
            self.writer = None;
        }

        if self.writer.is_none()
            && let Some(next) = self.queue.pop_front()
        {
            let mut writer = new_writer(next)?;
            writer.init(ring)?;
            self.writer = Some(writer);
        }

        Ok(())
    }

    pub(crate) fn enqueue_get_caps_lock(&mut self, ring: &mut IoUring) -> Result<()> {
        let resource = Box::new(CapsLock);
        if self.writer.is_none() {
            let mut writer = new_writer(resource)?;
            writer.init(ring)?;
            self.writer = Some(writer)
        } else {
            self.queue.push_back(resource);
        }
        Ok(())
    }

    pub(crate) fn dispatch(&mut self, cmd: String, ring: &mut IoUring) -> Result<()> {
        let resource = Box::new(Dispatch::new(cmd));
        if self.writer.is_none() {
            let mut writer = new_writer(resource)?;
            writer.init(ring)?;
            self.writer = Some(writer)
        } else {
            self.queue.push_back(resource);
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
