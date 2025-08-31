use crate::{Task as TaskView, View};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct SecondaryTask {
    title: String,
    created_at: u64,
    duration: u64,
}

impl SecondaryTask {
    fn upgrade(self, uuid: Uuid) -> PrimaryTask {
        PrimaryTask {
            title: self.title,
            uuid,
            created_at: self.created_at,
            persistent_duration: self.duration,
            tmp_started_at: None,
        }
    }

    fn serialize(&self, uuid: Uuid) -> TaskView {
        TaskView {
            title: self.title.clone().into(),
            uuid: uuid.to_string().into(),
            duration: format_duration(self.duration).into(),
            created_at: format_ts(self.created_at).into(),
            selected: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct PrimaryTask {
    title: String,
    uuid: Uuid,
    created_at: u64,
    persistent_duration: u64,
    tmp_started_at: Option<u64>,
}

impl PrimaryTask {
    fn new(title: String) -> Self {
        Self {
            title,
            uuid: Uuid::new_v4(),
            created_at: now(),
            persistent_duration: 0,
            tmp_started_at: None,
        }
    }

    fn is_running(&self) -> bool {
        self.tmp_started_at.is_some()
    }

    fn start(&mut self) {
        if self.tmp_started_at.is_none() {
            self.tmp_started_at = Some(now());
        }
    }

    fn stop(&mut self) {
        let Some(tmp) = self.tmp_started_at.as_mut() else {
            return;
        };

        self.persistent_duration += now() - *tmp;
        self.tmp_started_at = None;
    }

    fn toggle(&mut self) {
        if self.is_running() {
            self.stop();
        } else {
            self.start();
        }
    }

    fn downgrade(self) -> (Uuid, SecondaryTask) {
        (
            self.uuid,
            SecondaryTask {
                title: self.title,
                created_at: self.created_at,
                duration: self.persistent_duration,
            },
        )
    }

    fn serialize(&self) -> TaskView {
        let tmp = self.tmp_started_at.map(|t| now() - t).unwrap_or(0);

        TaskView {
            title: self.title.clone().into(),
            uuid: self.uuid.to_string().into(),
            duration: format_duration(self.persistent_duration + tmp)
                .to_string()
                .into(),
            created_at: format_ts(self.created_at).into(),
            selected: true,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct State {
    created_at: u64,
    primary: Option<PrimaryTask>,
    tasks: HashMap<Uuid, SecondaryTask>,
}

impl State {
    pub(crate) fn empty() -> Self {
        Self {
            created_at: now(),
            primary: None,
            tasks: HashMap::new(),
        }
    }

    pub(crate) fn created_at(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp(self.created_at as i64, 0)
            .expect("malformed state.created_at")
    }

    pub(crate) fn stop(&mut self) {
        if let Some(primary) = self.primary.as_mut() {
            primary.stop();
        }
    }

    pub(crate) fn toggle(&mut self) {
        if let Some(primary) = self.primary.as_mut() {
            primary.toggle();
        }
    }

    fn deselect(&mut self) {
        let Some(mut prev) = self.primary.take() else {
            return;
        };
        prev.stop();
        let (uuid, prev) = prev.downgrade();
        self.tasks.insert(uuid, prev);
    }

    pub(crate) fn select(&mut self, uuid: Uuid) {
        let Some(next) = self.tasks.remove(&uuid) else {
            log::error!("can't select task with UUID {uuid}: it does not exist");
            return;
        };
        let mut next = next.upgrade(uuid);

        let was_running = self.is_running();
        self.deselect();
        if was_running {
            next.start();
        }

        self.primary = Some(next);
    }

    pub(crate) fn add(&mut self, title: String) {
        self.deselect();
        let mut primary = PrimaryTask::new(title);
        primary.start();
        self.primary = Some(primary);
    }

    pub(crate) fn remove(&mut self, uuid: Uuid) {
        if self.primary.as_ref().is_some_and(|task| task.uuid == uuid) {
            self.primary = None;
        } else {
            self.tasks.remove(&uuid);
        }
    }

    pub(crate) fn is_running(&self) -> bool {
        self.primary.as_ref().is_some_and(|task| task.is_running())
    }

    pub(crate) fn serialize(&self) -> View {
        enum AnyTask<'a> {
            Primary(&'a PrimaryTask),
            Secondary(Uuid, &'a SecondaryTask),
        }
        impl AnyTask<'_> {
            fn created_at(&self) -> u64 {
                match self {
                    Self::Primary(task) => task.created_at,
                    Self::Secondary(_, task) => task.created_at,
                }
            }
        }

        let mut tasks = vec![];
        if let Some(primary) = self.primary.as_ref() {
            tasks.push(AnyTask::Primary(primary));
        }
        for (uuid, task) in self.tasks.iter() {
            tasks.push(AnyTask::Secondary(*uuid, task));
        }
        tasks.sort_unstable_by_key(|task| task.created_at());

        let tasks = tasks
            .into_iter()
            .map(|task| match task {
                AnyTask::Primary(primary) => primary.serialize(),
                AnyTask::Secondary(uuid, task) => task.serialize(uuid),
            })
            .collect::<Vec<_>>();

        View {
            created_at: format_ts(self.created_at).into(),
            tasks: tasks.into(),
            running: self.is_running(),
        }
    }
}

fn now() -> u64 {
    chrono::Utc::now().timestamp() as u64
}

fn format_duration(duration_in_seconds: u64) -> String {
    let (minutes, seconds) = (duration_in_seconds / 60, duration_in_seconds % 60);
    let (hours, minutes) = (minutes / 60, minutes % 60);
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn format_ts(ts: u64) -> String {
    chrono::DateTime::<Utc>::from_timestamp(ts as i64, 0)
        .expect("malformed timestamp")
        .naive_local()
        .to_string()
}
