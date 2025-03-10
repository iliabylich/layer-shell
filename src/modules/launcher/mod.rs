use crate::{Event, VerboseSender, modules::maybe_connected::MaybeConnected};
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
    tx: VerboseSender<Event>,
}

impl Launcher {
    pub(crate) fn new(
        tx: VerboseSender<Event>,
    ) -> (
        Self,
        MaybeConnected<Watcher<GlobalDir>>,
        MaybeConnected<Watcher<UserDir>>,
    ) {
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

        let global_watcher = Watcher::<GlobalDir>::new(Rc::clone(&state), tx.clone());
        let user_watcher = Watcher::<UserDir>::new(Rc::clone(&state), tx.clone());

        (Self { state, tx }, global_watcher, user_watcher)
    }

    pub(crate) fn reset(&mut self) {
        let mut state = self.state.borrow_mut();
        let event = state.reset();
        self.tx.send(event);
    }

    pub(crate) fn go_up(&mut self) {
        let mut state = self.state.borrow_mut();
        let event = state.go_up();
        self.tx.send(event);
    }

    pub(crate) fn go_down(&mut self) {
        let mut state = self.state.borrow_mut();
        let event = state.go_down();
        self.tx.send(event);
    }

    pub(crate) fn set_search(&mut self, search: String) {
        let mut state = self.state.borrow_mut();
        let event = state.set_search(search);
        self.tx.send(event);
    }

    pub(crate) fn exec_selected(&mut self) {
        let state = self.state.borrow();
        if let Err(err) = state.exec_selected() {
            log::error!("{:?}", err);
        }
    }
}
