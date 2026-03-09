pub(crate) use reader::HyprlandReader;
pub(crate) use writer::HyprlandWriter;

pub use state::HyprlandWorkspace;

use crate::{event_queue::EventQueue, unix_socket::new_unix_socket};
use state::HyprlandState;
use std::{cell::RefCell, rc::Rc};

mod reader;
mod resources;
mod state;
mod writer;

pub(crate) struct Hyprland;

impl Hyprland {
    pub(crate) fn connect(events: EventQueue) -> (Option<HyprlandReader>, Option<HyprlandWriter>) {
        let xdg_runtime_dir = match std::env::var("XDG_RUNTIME_DIR") {
            Ok(var) => var,
            Err(err) => {
                log::error!("{err:?}");
                return (None, None);
            }
        };

        let hyprland_instance_signature = match std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
            Ok(var) => var,
            Err(err) => {
                log::error!("{err:?}");
                return (None, None);
            }
        };

        let reader_addr = new_unix_socket(
            format!("{xdg_runtime_dir}/hypr/{hyprland_instance_signature}/.socket2.sock")
                .as_bytes(),
        );

        let writer_addr = new_unix_socket(
            format!("{xdg_runtime_dir}/hypr/{hyprland_instance_signature}/.socket.sock").as_bytes(),
        );

        let state = Rc::new(RefCell::new(HyprlandState::empty()));

        (
            Some(HyprlandReader::new(
                reader_addr,
                Rc::clone(&state),
                events.clone(),
            )),
            Some(HyprlandWriter::new(
                writer_addr,
                Rc::clone(&state),
                events.clone(),
            )),
        )
    }
}
