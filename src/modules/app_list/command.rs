use crate::modules::app_list::state::State;

pub(crate) fn reset() {
    State::instance().reset();
}

pub(crate) fn go_up() {
    State::instance().go_up();
}

pub(crate) fn go_down() {
    State::instance().go_down();
}

pub(crate) fn set_search(s: *const u8) {
    let string = unsafe { std::ffi::CStr::from_ptr(s.cast()) };
    let string = string.to_str().unwrap().to_string();
    State::instance().set_search(string);
}

pub(crate) fn exec_selected() {
    State::instance().exec_selected();
}
