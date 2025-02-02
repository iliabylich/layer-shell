use crate::Command;
pub(crate) use action::Action;
use anyhow::Result;
use std::{ops::ControlFlow, time::Duration};

mod action;

pub(crate) trait Actor: Send + std::fmt::Debug {
    fn name() -> &'static str
    where
        Self: Sized;
    fn start() -> Result<Box<dyn Actor>>
    where
        Self: Sized;

    fn tick(&mut self) -> Result<ControlFlow<(), Duration>>;

    fn exec(&mut self, _: &Command) -> Result<ControlFlow<()>>;
}
