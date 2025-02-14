use crate::{
    event::App,
    modules::launcher::{desktop_file::DesktopFile, watcher::WatcherUpdate},
    Event,
};
use anyhow::Result;
use std::collections::HashMap;

pub(crate) struct State {
    selected_idx: usize,
    path_to_desktop_files: HashMap<String, DesktopFile>,
    pattern: String,
}

impl State {
    const MAX_ITEMS: usize = 5;

    pub(crate) fn new(desktop_files: Vec<DesktopFile>) -> (Self, Event) {
        let mut path_to_desktop_files = HashMap::new();
        for desktop_file in desktop_files {
            path_to_desktop_files.insert(desktop_file.path.clone(), desktop_file);
        }

        let this = Self {
            selected_idx: 0,
            path_to_desktop_files,
            pattern: String::new(),
        };
        let event = this.as_event();
        (this, event)
    }

    pub(crate) fn go_up(&mut self) -> Event {
        self.selected_idx = self.selected_idx.saturating_sub(1);
        self.as_event()
    }

    pub(crate) fn go_down(&mut self) -> Event {
        self.selected_idx = std::cmp::min(Self::MAX_ITEMS - 1, self.selected_idx + 1);
        self.as_event()
    }

    pub(crate) fn set_search(&mut self, pattern: String) -> Event {
        self.selected_idx = 0;
        self.pattern = pattern;
        self.as_event()
    }

    pub(crate) fn exec_selected(&self) -> Result<()> {
        if let Some(desktop_file) = self.visible().get(self.selected_idx) {
            desktop_file.exec()?;
        }
        Ok(())
    }

    pub(crate) fn process_watcher_update(&mut self, update: WatcherUpdate) -> Event {
        for desktop_file in DesktopFile::parse_many(update.created_or_updated.into_iter()) {
            self.path_to_desktop_files
                .insert(desktop_file.path.clone(), desktop_file);
        }
        for path in update.removed {
            self.path_to_desktop_files.remove(&path);
        }
        self.reset()
    }

    pub(crate) fn reset(&mut self) -> Event {
        self.pattern = String::new();
        self.selected_idx = 0;
        self.as_event()
    }

    fn as_event(&self) -> Event {
        let apps = self
            .visible()
            .into_iter()
            .enumerate()
            .map(|(idx, desktop_file)| App {
                name: desktop_file.app_name.into(),
                selected: idx == self.selected_idx,
                icon: desktop_file.icon,
            })
            .collect::<Vec<_>>();

        Event::Launcher { apps: apps.into() }
    }

    fn visible(&self) -> Vec<DesktopFile> {
        let pattern = self.pattern.to_lowercase();

        let mut desktop_files = self
            .path_to_desktop_files
            .values()
            .filter(|desktop_file| desktop_file.app_name.to_lowercase().contains(&pattern))
            .cloned()
            .collect::<Vec<_>>();
        desktop_files.sort_unstable_by(|file1, file2| file1.app_name.cmp(&file2.app_name));
        desktop_files.truncate(Self::MAX_ITEMS);
        desktop_files
    }
}
