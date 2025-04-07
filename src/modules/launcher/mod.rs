use crate::channel::EventSender0;
use desktop_file::DesktopFile;
use dir::{GlobalDir, UserDir, WatcherDir as _};
use state::State;
use std::{cell::RefCell, rc::Rc};
use watcher::Watcher;

mod desktop_file;
mod dir;
mod state;
mod watcher;

pub(crate) struct Launcher {
    state: Rc<RefCell<State>>,
    tx: EventSender0,
}

impl Launcher {
    pub(crate) fn new(tx: &EventSender0) -> Self {
        let mut filelist = vec![];
        if let Ok(dir) = GlobalDir::new() {
            match dir::glob(&dir) {
                Ok(mut files) => filelist.append(&mut files),
                Err(err) => log::error!("{:?}", err),
            }
        }
        if let Ok(dir) = UserDir::new() {
            match dir::glob(&dir) {
                Ok(mut files) => filelist.append(&mut files),
                Err(err) => log::error!("{:?}", err),
            }
        }

        let desktop_files = DesktopFile::parse_many(filelist.iter());
        let (state, event) = State::new(desktop_files);
        tx.send(event);

        let state = Rc::new(RefCell::new(state));
        Self {
            state,
            tx: tx.clone(),
        }
    }

    pub(crate) fn reset(&mut self) {
        let mut state = self.state.borrow_mut();
        state.reset();
        let event = state.as_event();
        self.tx.send(event);
    }

    pub(crate) fn go_up(&mut self) {
        let mut state = self.state.borrow_mut();
        state.go_up();
        let event = state.as_event();
        self.tx.send(event);
    }

    pub(crate) fn go_down(&mut self) {
        let mut state = self.state.borrow_mut();
        state.go_down();
        let event = state.as_event();
        self.tx.send(event);
    }

    pub(crate) fn set_search(&mut self, search: String) {
        let mut state = self.state.borrow_mut();
        state.set_search(search);
        let event = state.as_event();
        self.tx.send(event);
    }

    pub(crate) fn exec_selected(&mut self) {
        let state = self.state.borrow();
        if let Err(err) = state.exec_selected() {
            log::error!("{:?}", err);
        }
    }
}

pub(crate) type GlobalLauncherWatcher = Watcher<GlobalDir>;
pub(crate) type UserLauncherWatcher = Watcher<UserDir>;
