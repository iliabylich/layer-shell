use crate::{scheduler::Actor, Command, Event};
use anyhow::{Context, Result};
use desktop_file::DesktopFile;
use state::State;
use std::{ops::ControlFlow, sync::mpsc::Sender, time::Duration};
use watcher::{Watcher, WatcherUpdate};

mod desktop_file;
mod dirs;
mod state;
mod watcher;

#[derive(Debug)]
pub(crate) struct AppList {
    state: State,
    watchers: Vec<Watcher>,
}

impl Actor for AppList {
    fn name() -> &'static str {
        "AppList"
    }

    fn start(tx: Sender<Event>) -> Result<Box<dyn Actor>> {
        let dirlist = dirs::dirlist();

        let mut watchers = vec![];
        for dir in dirlist.iter() {
            match Watcher::new(dir) {
                Ok(watcher) => watchers.push(watcher),
                Err(err) => log::error!("{:?}", err),
            }
        }

        let filelist = dirs::filelist(&dirlist).context("failed to read filelist")?;
        let desktop_apps = DesktopFile::parse_many(filelist.iter());

        Ok(Box::new(Self {
            state: State::new(tx, desktop_apps),
            watchers,
        }))
    }

    fn tick(&mut self) -> Result<ControlFlow<(), Duration>> {
        if self.watchers.iter().all(|w| !w.is_enabled()) {
            return Ok(ControlFlow::Break(()));
        }

        let mut update = WatcherUpdate::new_empty();

        loop {
            let mut polled = false;
            for watcher in self.watchers.iter_mut() {
                if let Some(buf) = watcher.poll() {
                    update.merge(buf);
                    polled = true;
                }
            }
            if !polled {
                break;
            }
        }

        if !update.is_empty() {
            self.state.process_watcher_update(update);
        }

        Ok(ControlFlow::Continue(Duration::from_millis(100)))
    }

    fn exec(&mut self, cmd: &Command) -> Result<ControlFlow<()>> {
        match cmd {
            Command::AppListReset => self.state.reset(),
            Command::AppListGoUp => self.state.go_up(),
            Command::AppListGoDown => self.state.go_down(),
            Command::AppListSetSearch { search } => self.state.set_search(search.clone()),
            Command::AppListExecSelected => self.state.exec_selected()?,

            _ => {}
        }
        Ok(ControlFlow::Continue(()))
    }
}
