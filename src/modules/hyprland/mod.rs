pub(crate) use queue::HyprlandQueue;
pub(crate) use reader::HyprlandReader;
pub(crate) use writer::HyprlandWriter;

pub use state::HyprlandWorkspace;

use crate::{sansio::UnixSocketReader, unix_socket::new_unix_socket};

mod queue;
mod reader;
mod resources;
mod state;
mod writer;

pub(crate) struct Hyprland;

impl Hyprland {
    pub(crate) fn connect() -> (HyprlandReader, HyprlandWriter) {
        HyprlandQueue::init();

        let xdg_runtime_dir = match std::env::var("XDG_RUNTIME_DIR") {
            Ok(var) => var,
            Err(err) => {
                log::error!("{err:?}");
                HyprlandQueue::make_dummy();

                return (
                    HyprlandReader::new(UnixSocketReader::dummy()),
                    HyprlandWriter::dummy(),
                );
            }
        };

        let hyprland_instance_signature = match std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
            Ok(var) => var,
            Err(err) => {
                log::error!("{err:?}");
                HyprlandQueue::make_dummy();

                return (
                    HyprlandReader::new(UnixSocketReader::dummy()),
                    HyprlandWriter::dummy(),
                );
            }
        };

        let reader_addr = new_unix_socket(
            format!("{xdg_runtime_dir}/hypr/{hyprland_instance_signature}/.socket2.sock")
                .as_bytes(),
        );

        let writer_addr = new_unix_socket(
            format!("{xdg_runtime_dir}/hypr/{hyprland_instance_signature}/.socket.sock").as_bytes(),
        );

        (
            HyprlandReader::new(UnixSocketReader::new(reader_addr)),
            HyprlandWriter::new(writer_addr),
        )
    }
}
