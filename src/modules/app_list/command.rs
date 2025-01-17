use crate::modules::app_list::state::State;

pub(crate) fn reset() {
    State::reset();
}

pub(crate) fn go_up() {
    State::go_up();
}

pub(crate) fn go_down() {
    State::go_down();
}

pub(crate) fn set_search(s: *const u8) {
    let string = unsafe { std::ffi::CStr::from_ptr(s.cast()) };
    let Ok(string) = string.to_str() else {
        log::error!("invalid search pattern");
        return;
    };
    State::set_search(string.to_string());
}

pub(crate) fn exec_selected() {
    State::exec_selected();
}
