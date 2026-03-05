pub(crate) use reader::HyprlandReader;
use state::HyprlandState;
pub use state::HyprlandWorkspace;
use std::{cell::RefCell, rc::Rc};
pub(crate) use writer::HyprlandWriter;

mod oneshot_writer;
mod reader;
mod resources;
mod state;
mod writer;

pub(crate) struct Hyprland;

impl Hyprland {
    pub(crate) fn new() -> (Option<HyprlandReader>, Option<HyprlandWriter>) {
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

        let state = Rc::new(RefCell::new(HyprlandState::empty()));

        let writer = HyprlandWriter::new(
            &xdg_runtime_dir,
            &hyprland_instance_signature,
            Rc::clone(&state),
        );
        let reader = HyprlandReader::new(
            &xdg_runtime_dir,
            &hyprland_instance_signature,
            Rc::clone(&state),
        );

        (Some(reader), Some(writer))
    }
}
