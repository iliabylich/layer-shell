use crate::{Event, modules::hyprland::writer::CapsLock};
use anyhow::Result;
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

fn new_reader() -> Box<HyprlandReader> {
    HyprlandReader::new()
}
fn new_writer(resource: Box<dyn WriterResource>) -> Box<HyprlandWriter> {
    HyprlandWriter::new(resource)
}

impl Hyprland {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            reader: new_reader(),
            writer: Some(new_writer(Box::new(WorkspaceListResource))),
            state: HyprlandState::default(),
            queue: VecDeque::from([
                Box::new(ActiveWorkspaceResource) as Box<dyn WriterResource>,
                Box::new(DevicesResource),
            ]),
        })
    }

    pub(crate) fn init(&mut self) {
        self.reader.init();
        if let Some(writer) = self.writer.as_mut() {
            writer.init();
        }
    }

    pub(crate) fn process_reader(
        &mut self,
        op: u8,
        res: i32,
        events: &mut Vec<Event>,
    ) -> Result<()> {
        let mut hevents = vec![];
        self.reader.process(op, res, &mut hevents)?;
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
        events: &mut Vec<Event>,
    ) -> Result<()> {
        if let Some(writer) = self.writer.as_mut()
            && let Some(reply) = writer.process(op, res)?
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
            let mut writer = new_writer(next);
            writer.init();
            self.writer = Some(writer);
        }

        Ok(())
    }

    pub(crate) fn enqueue_get_caps_lock(&mut self) {
        let resource = Box::new(CapsLock);
        if self.writer.is_none() {
            let mut writer = new_writer(resource);
            writer.init();
            self.writer = Some(writer)
        } else {
            self.queue.push_back(resource);
        }
    }

    pub(crate) fn dispatch(&mut self, cmd: String) {
        let resource = Box::new(Dispatch::new(cmd));
        if self.writer.is_none() {
            let mut writer = new_writer(resource);
            writer.init();
            self.writer = Some(writer)
        } else {
            self.queue.push_back(resource);
        }
    }
}

fn xdg_runtime_dir() -> String {
    std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
        eprintln!("no XDG_RUNTIME_DIR variable");
        std::process::exit(1)
    })
}

fn hyprland_instance_signature() -> String {
    std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap_or_else(|_| {
        eprintln!("no HYPRLAND_INSTANCE_SIGNATURE, are you in Hyprland?");
        std::process::exit(1);
    })
}
