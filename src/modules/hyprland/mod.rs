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
    writer: Box<HyprlandWriter>,
    writer_is_ready: bool,
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
            writer_is_ready: false,
            state: HyprlandState::default(),
            queue: VecDeque::from([
                Box::new(ActiveWorkspaceResource) as Box<dyn WriterResource>,
                Box::new(DevicesResource),
            ]),
        }))
    }

    pub(crate) fn init(&mut self, ring: &mut IoUring) -> Result<()> {
        self.reader.init(ring)?;
        self.writer.init(ring)?;
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
        if let Some(reply) = self.writer.process(op, res, ring)? {
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

            self.writer_is_ready = true;
            self.start_new_writer_if_ready(ring)?;
        }
        Ok(())
    }

    fn start_new_writer_if_ready(&mut self, ring: &mut IoUring) -> Result<()> {
        if !self.writer_is_ready {
            return Ok(());
        }

        if let Some(next) = self.queue.pop_front() {
            self.writer = writer(next)?;
            self.writer.init(ring)?;
        }

        Ok(())
    }

    pub(crate) fn enqueue_get_caps_lock(&mut self, ring: &mut IoUring) -> Result<()> {
        self.queue.push_back(Box::new(CapsLock));
        self.start_new_writer_if_ready(ring)?;
        Ok(())
    }

    pub(crate) fn dispatch(&mut self, cmd: String, ring: &mut IoUring) -> Result<()> {
        self.queue.push_back(Box::new(Dispatch::new(cmd)));
        self.start_new_writer_if_ready(ring)?;
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
