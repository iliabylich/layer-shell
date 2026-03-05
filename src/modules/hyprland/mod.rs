use crate::Event;
use anyhow::Result;
use queue_writer::QueueWriter;
use reader::HyprlandReader;
use resources::{
    ActiveWorkspaceResource, CapsLockResource, DevicesResource, DispatchResource,
    WorkspacesResource,
};
use state::HyprlandState;
pub use state::HyprlandWorkspace;

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

        let writer = QueueWriter::new(&xdg_runtime_dir, &hyprland_instance_signature);
        let reader = HyprlandReader::new(&xdg_runtime_dir, &hyprland_instance_signature);

        Some(Box::new(Self {
            reader,
            writer,
            state: HyprlandState::empty(),
        }))
    }

    pub(crate) fn init(&mut self) {
        self.reader.init();

        self.writer.enqueue(Box::new(WorkspacesResource));
        self.writer.enqueue(Box::new(ActiveWorkspaceResource));
        self.writer.enqueue(Box::new(DevicesResource));
    }

    pub(crate) fn process_reader(&mut self, op: u8, res: i32, events: &mut Vec<Event>) {
        match self.reader.process(op, res) {
            Ok(diffs) => {
                for diff in diffs {
                    if let Some(event) = self.state.apply(diff) {
                        events.push(event);
                    }
                }
            }
            Err(err) => {
                log::error!("Hyprland reader: {err:?}");
            }
        }
    }

    fn try_process_writer(&mut self, op: u8, res: i32, events: &mut Vec<Event>) -> Result<()> {
        let Some(diff) = self.writer.process(op, res)? else {
            return Ok(());
        };

        if let Some(event) = self.state.apply(diff) {
            events.push(event);
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

    pub(crate) fn enqueue_dispatch(&mut self, cmd: String) {
        self.writer.enqueue(Box::new(DispatchResource::new(cmd)));
    }
}
