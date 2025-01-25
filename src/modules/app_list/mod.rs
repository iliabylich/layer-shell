use anyhow::Result;

mod command;
mod state;
mod system_app;

pub(crate) struct AppList;

impl AppList {
    pub(crate) fn exec_selected() -> Result<()> {
        command::exec_selected()
    }

    pub(crate) fn go_down() -> Result<()> {
        command::go_down()
    }
    pub(crate) fn go_up() -> Result<()> {
        command::go_up()
    }
    pub(crate) fn reset() -> Result<()> {
        command::reset()
    }
    pub(crate) fn set_search(s: *const u8) -> Result<()> {
        command::set_search(s)
    }
}
