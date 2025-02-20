use crate::{
    epoll::{FdId, Reader},
    modules::{
        launcher::{dir::WatcherDir, state::State},
        maybe_connected::MaybeConnected,
    },
    Event, VerboseSender,
};
use anyhow::{bail, Context as _, Result};
use inotify::{EventMask, Inotify, WatchMask};
use std::{cell::RefCell, collections::HashSet, io::ErrorKind, os::fd::AsRawFd, rc::Rc};

pub(crate) struct Watcher<T: WatcherDir> {
    dir: T,
    inotify: Inotify,
    state: Rc<RefCell<State>>,
    tx: VerboseSender<Event>,
}

impl<T> Watcher<T>
where
    T: WatcherDir,
{
    fn try_new(state: Rc<RefCell<State>>, tx: VerboseSender<Event>) -> Result<Self> {
        let dir = T::new()?;

        let inotify = Inotify::init().context("failed to initialize Inotify")?;

        inotify
            .watches()
            .add(
                dir.path(),
                WatchMask::CREATE | WatchMask::DELETE | WatchMask::MODIFY,
            )
            .with_context(|| format!("Failed to add file watch for dir {}", dir.path()))?;

        Ok(Self {
            dir,
            inotify,
            state,
            tx,
        })
    }

    pub(crate) fn new(state: Rc<RefCell<State>>, tx: VerboseSender<Event>) -> MaybeConnected<Self> {
        MaybeConnected::new(Self::try_new(state, tx))
    }
}

#[derive(Debug)]
pub(crate) struct WatcherUpdate {
    pub(crate) created_or_updated: HashSet<String>,
    pub(crate) removed: HashSet<String>,
}

impl<T> Reader for Watcher<T>
where
    T: WatcherDir,
{
    type Output = ();

    const NAME: &str = "Inotify Watcher";

    fn read(&mut self) -> Result<Self::Output> {
        let mut buffer = [0; 1024];
        let mut created_or_updated = HashSet::new();
        let mut removed = HashSet::new();

        match self.inotify.read_events(&mut buffer) {
            Ok(events) => {
                for event in events {
                    if let Some(name) = event.name.and_then(|name| name.to_str()) {
                        let path = format!("{}/{}", self.dir.path(), name);
                        if event.mask.intersects(EventMask::CREATE | EventMask::MODIFY) {
                            created_or_updated.insert(path);
                        } else if event.mask.intersects(EventMask::DELETE) {
                            removed.insert(path);
                        }
                    }
                }
            }

            Err(err) if err.kind() == ErrorKind::WouldBlock => return Ok(()),
            Err(err) => {
                bail!(
                    "failed to read events of dir {}: {:?}",
                    self.dir.path(),
                    err
                );
            }
        }

        let mut state = self.state.borrow_mut();
        let event = state.process_watcher_update(WatcherUpdate {
            created_or_updated,
            removed,
        });
        self.tx.send(event);

        Ok(())
    }

    fn fd(&self) -> i32 {
        self.inotify.as_raw_fd()
    }

    fn fd_id(&self) -> FdId {
        self.dir.fd_id()
    }
}
