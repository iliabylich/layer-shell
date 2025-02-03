use anyhow::{Context as _, Result};
use inotify::{EventMask, Inotify, WatchMask};
use std::{collections::HashSet, io::ErrorKind};

#[derive(Debug)]
pub(crate) struct Watcher {
    dir: String,
    inotify: Inotify,
    enabled: bool,
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
            enabled: true,
        })
    }

    pub(crate) fn poll(&mut self) -> Option<WatcherUpdate> {
        if !self.enabled {
            return None;
        }

        let mut buffer = [0; 1024];
        let mut created_or_updated_paths = HashSet::new();
        let mut removed_paths = HashSet::new();

        match self.inotify.read_events(&mut buffer) {
            Ok(events) => {
                for event in events {
                    if let Some(name) = event.name.and_then(|name| name.to_str()) {
                        let path = format!("{}/{}", self.dir, name);
                        if event.mask.intersects(EventMask::CREATE | EventMask::MODIFY) {
                            created_or_updated_paths.insert(path);
                        } else if event.mask.intersects(EventMask::DELETE) {
                            removed_paths.insert(path);
                        }
                    }
                }
            }

            Err(err) if err.kind() == ErrorKind::WouldBlock => return None,
            Err(err) => {
                log::error!("failed to read events of {:?}: {:?}, disabling", self, err);
                self.enabled = false;
                return None;
            }
        }

        Some(WatcherUpdate {
            created_or_updated_paths,
            removed_paths,
        })
    }

    pub(crate) fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Debug)]
pub(crate) struct WatcherUpdate {
    pub(crate) created_or_updated_paths: HashSet<String>,
    pub(crate) removed_paths: HashSet<String>,
}

impl WatcherUpdate {
    pub(crate) fn new_empty() -> Self {
        Self {
            created_or_updated_paths: HashSet::new(),
            removed_paths: HashSet::new(),
        }
    }

    pub(crate) fn merge(&mut self, other: Self) {
        self.created_or_updated_paths
            .extend(other.created_or_updated_paths);
        self.removed_paths.extend(other.removed_paths);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.created_or_updated_paths.is_empty() && self.removed_paths.is_empty()
    }
}
