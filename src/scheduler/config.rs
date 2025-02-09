use std::sync::mpsc::Sender;

use crate::{scheduler::Actor, Event};
use anyhow::Result;

pub(crate) struct ActorConfig {
    pub(crate) name: &'static str,
    pub(crate) start: fn(Sender<Event>) -> Result<Box<dyn Actor>>,
}

pub(crate) struct Config {
    modules_to_start: Vec<ActorConfig>,
}

impl Config {
    pub(crate) fn new() -> Self {
        Self {
            modules_to_start: vec![],
        }
    }

    pub(crate) fn add<T: Actor>(&mut self) {
        let module = ActorConfig {
            name: T::name(),
            start: T::start,
        };
        self.modules_to_start.push(module);
    }

    pub(crate) fn into_iter(self) -> impl Iterator<Item = ActorConfig> {
        self.modules_to_start.into_iter()
    }
}
