use crate::{Event, VerboseSender};
use desktop_file::DesktopFile;
use state::State;
use watcher::Watcher;

mod desktop_file;
mod dirs;
mod state;
mod watcher;

pub(crate) struct Launcher {
    state: State,
    global_dir_watcher: Option<Watcher>,
    user_dir_watcher: Option<Watcher>,
    tx: VerboseSender<Event>,
}

impl Launcher {
    pub(crate) fn new(tx: VerboseSender<Event>) -> Self {
        let mut filelist = vec![];

        let global_dir_watcher =
            dirs::global_dir().and_then(|dir| make_watcher_and_parse_filelist(dir, &mut filelist));

        let user_dir_watcher =
            dirs::user_dir().and_then(|dir| make_watcher_and_parse_filelist(dir, &mut filelist));

        let desktop_apps = DesktopFile::parse_many(filelist.iter());
        let (state, event) = State::new(desktop_apps);
        tx.send(event);

        Self {
            state,
            global_dir_watcher,
            user_dir_watcher,
            tx,
        }
    }

    pub(crate) fn global_inotify_fd(&self) -> Option<i32> {
        self.global_dir_watcher.as_ref().map(|w| w.fd())
    }

    pub(crate) fn user_inotify_fd(&self) -> Option<i32> {
        self.user_dir_watcher.as_ref().map(|w| w.fd())
    }

    pub(crate) fn read_global(&mut self) {
        if let Some(watcher) = self.global_dir_watcher.as_mut() {
            if let Some(update) = watcher.poll() {
                let event = self.state.process_watcher_update(update);
                self.tx.send(event);
            }
        }
    }

    pub(crate) fn read_user(&mut self) {
        if let Some(watcher) = self.user_dir_watcher.as_mut() {
            if let Some(update) = watcher.poll() {
                let event = self.state.process_watcher_update(update);
                self.tx.send(event);
            }
        }
    }

    pub(crate) fn reset(&mut self) {
        let event = self.state.reset();
        self.tx.send(event);
    }

    pub(crate) fn go_up(&mut self) {
        let event = self.state.go_up();
        self.tx.send(event);
    }

    pub(crate) fn go_down(&mut self) {
        let event = self.state.go_down();
        self.tx.send(event);
    }

    pub(crate) fn set_search(&mut self, search: String) {
        let event = self.state.set_search(search);
        self.tx.send(event);
    }

    pub(crate) fn exec_selected(&mut self) {
        if let Err(err) = self.state.exec_selected() {
            log::error!("{:?}", err);
        }
    }
}

fn make_watcher_and_parse_filelist(dir: String, filelist: &mut Vec<String>) -> Option<Watcher> {
    match dirs::glob(&dir) {
        Ok(mut paths) => filelist.append(&mut paths),
        Err(err) => log::error!("{:?}", err),
    }

    match Watcher::new(&dir) {
        Ok(watcher) => Some(watcher),
        Err(err) => {
            log::error!("{:?}", err);
            None
        }
    }
}
