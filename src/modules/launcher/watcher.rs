use anyhow::{Context as _, Result};
use inotify::{EventMask, Inotify, WatchMask};
use std::{collections::HashSet, io::ErrorKind, os::fd::AsRawFd};

#[derive(Debug)]
pub(crate) struct Watcher {
    dir: String,
    inotify: Inotify,
}

impl Watcher {
    pub(crate) fn new(dir: &str) -> Result<Self> {
        let inotify = Inotify::init().context("failed to initialize Inotify")?;

        inotify
            .watches()
            .add(
                dir,
                WatchMask::CREATE | WatchMask::DELETE | WatchMask::MODIFY,
            )
            .with_context(|| format!("Failed to add file watch for dir {dir}"))?;

        Ok(Self {
            dir: dir.to_string(),
            inotify,
        })
    }

    pub(crate) fn poll(&mut self) -> Option<WatcherUpdate> {
        let mut buffer = [0; 1024];
        let mut created_or_updated = HashSet::new();
        let mut removed = HashSet::new();

        match self.inotify.read_events(&mut buffer) {
            Ok(events) => {
                for event in events {
                    if let Some(name) = event.name.and_then(|name| name.to_str()) {
                        let path = format!("{}/{}", self.dir, name);
                        if event.mask.intersects(EventMask::CREATE | EventMask::MODIFY) {
                            created_or_updated.insert(path);
                        } else if event.mask.intersects(EventMask::DELETE) {
                            removed.insert(path);
                        }
                    }
                }
            }

            Err(err) if err.kind() == ErrorKind::WouldBlock => return None,
            Err(err) => {
                log::error!("failed to read events of {:?}: {:?}", self, err);
                return None;
            }
        }

        Some(WatcherUpdate {
            created_or_updated,
            removed,
        })
    }

    pub(crate) fn fd(&self) -> i32 {
        self.inotify.as_raw_fd()
    }
}

#[derive(Debug)]
pub(crate) struct WatcherUpdate {
    pub(crate) created_or_updated: HashSet<String>,
    pub(crate) removed: HashSet<String>,
}
