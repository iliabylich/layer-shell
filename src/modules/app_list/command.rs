use crate::modules::app_list::State;

pub(crate) async fn reset() {
    State::instance().reset().await;
}

pub(crate) async fn go_up() {
    State::instance().go_up().await;
}

pub(crate) async fn go_down() {
    State::instance().go_down().await;
}

pub(crate) async fn set_search(s: *const u8) {
    let string = unsafe { std::ffi::CStr::from_ptr(s.cast()) };
    let string = string.to_str().unwrap().to_string();
    State::instance().set_search(string).await;
}

pub(crate) async fn exec_selected() {
    State::instance().exec_selected().await;
}
