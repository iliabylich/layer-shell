use crate::Event;
use anyhow::Result;
use queue_writer::QueueWriter;
use reader::HyprlandReader;
use resources::{
    ActiveWorkspaceResource, CapsLockResource, DevicesResource, DispatchResource,
    WorkspacesResource, WriterReply,
};
use state::HyprlandState;
pub use state::HyprlandWorkspace;

mod event;
mod oneshot_writer;
mod queue_writer;
mod reader;
mod resources;
mod state;

pub(crate) struct Hyprland {
    reader: Box<HyprlandReader>,
    writer: QueueWriter,
    state: HyprlandState,
}

fn new_reader() -> Box<HyprlandReader> {
    HyprlandReader::new()
}

impl Hyprland {
    pub(crate) fn new() -> Option<Box<Self>> {
        let xdg_runtime_dir = match std::env::var("XDG_RUNTIME_DIR") {
            Ok(var) => var,
            Err(err) => {
                log::error!("{err:?}");
                return None;
            }
        };

        let hyprland_instance_signature = match std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
            Ok(var) => var,
            Err(err) => {
                log::error!("{err:?}");
                return None;
            }
        };

        let writer = QueueWriter::new(xdg_runtime_dir, hyprland_instance_signature);

        Some(Box::new(Self {
            reader: new_reader(),
            writer,
            state: HyprlandState::default(),
        }))
    }

    pub(crate) fn init(&mut self) {
        self.reader.init();

        self.writer.enqueue(Box::new(WorkspacesResource));
        self.writer.enqueue(Box::new(ActiveWorkspaceResource));
        self.writer.enqueue(Box::new(DevicesResource));
    }

    pub(crate) fn process_reader(&mut self, op: u8, res: i32, events: &mut Vec<Event>) {
        let mut hevents = vec![];
        self.reader.process(op, res, &mut hevents);
        for hevent in hevents {
            let event = self.state.apply(hevent);
            events.push(event);
        }
    }

    fn try_process_writer(&mut self, op: u8, res: i32, events: &mut Vec<Event>) -> Result<()> {
        let Some(reply) = self.writer.process(op, res)? else {
            return Ok(());
        };

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

        Ok(())
    }

    pub(crate) fn process_writer(&mut self, op: u8, res: i32, events: &mut Vec<Event>) {
        if let Err(err) = self.try_process_writer(op, res, events) {
            log::error!("{err:?}")
        }
    }

    pub(crate) fn enqueue_get_caps_lock(&mut self) {
        self.writer.enqueue(Box::new(CapsLockResource));
    }

    pub(crate) fn dispatch(&mut self, cmd: String) {
        self.writer.enqueue(Box::new(DispatchResource::new(cmd)));
    }
}

fn xdg_runtime_dir() -> Option<String> {
    match std::env::var("XDG_RUNTIME_DIR") {
        Ok(ok) => Some(ok),
        Err(_) => {
            log::error!("no XDG_RUNTIME_DIR variable");
            None
        }
    }
}

fn hyprland_instance_signature() -> Option<String> {
    match std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        Ok(ok) => Some(ok),
        Err(_) => {
            log::error!("no HYPRLAND_INSTANCE_SIGNATURE, are you in Hyprland?");
            None
        }
    }
}
