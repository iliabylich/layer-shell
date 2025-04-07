use crate::{
    channel::EventSender0,
    fd_id::FdId,
    modules::{
        Module,
        launcher::{Launcher, dir::WatcherDir, state::State},
    },
};
use anyhow::{Context as _, Result, bail};
use inotify::{EventMask, Inotify, WatchMask};
use std::{
    cell::RefCell,
    collections::HashSet,
    io::ErrorKind,
    os::fd::{AsRawFd, RawFd},
    rc::Rc,
};

pub(crate) struct Watcher<T: WatcherDir> {
    dir: T,
    inotify: Inotify,
    state: Option<Rc<RefCell<State>>>,
    tx: EventSender0,
}

impl<T: WatcherDir> Module for Watcher<T> {
    const FD_ID: FdId = T::FD_ID;
    const NAME: &str = T::NAME;

    type ReadOutput = ();

    fn new(tx: EventSender0) -> Result<Self> {
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
            state: None,
            tx,
        })
    }

    fn read_events(&mut self) -> Result<()> {
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

        let mut state = self
            .state
            .as_mut()
            .context("state is not set")?
            .borrow_mut();
        state.process_watcher_update(WatcherUpdate {
            created_or_updated,
            removed,
        });
        let event = state.as_event();
        self.tx.send(event);

        Ok(())
    }
}

impl<T> AsRawFd for Watcher<T>
where
    T: WatcherDir,
{
    fn as_raw_fd(&self) -> RawFd {
        self.inotify.as_raw_fd()
    }
}

impl<T> Watcher<T>
where
    T: WatcherDir,
{
    pub(crate) fn connect(&mut self, launcher: &Launcher) {
        self.state = Some(Rc::clone(&launcher.state));
    }
}

#[derive(Debug)]
pub(crate) struct WatcherUpdate {
    pub(crate) created_or_updated: HashSet<String>,
    pub(crate) removed: HashSet<String>,
}
