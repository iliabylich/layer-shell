use anyhow::{Context as _, Result};

use crate::modules::app_list::state::State;

pub(crate) fn reset() -> Result<()> {
    State::reset()
}

pub(crate) fn go_up() -> Result<()> {
    State::go_up()
}

pub(crate) fn go_down() -> Result<()> {
    State::go_down()
}

pub(crate) fn set_search(s: *const u8) -> Result<()> {
    let string = unsafe { std::ffi::CStr::from_ptr(s.cast()) };
    let string = string.to_str().context("invalid search pattern")?;
    State::set_search(string.to_string())
}

pub(crate) fn exec_selected() -> Result<()> {
    State::exec_selected()
}
