use crate::Command;
pub(crate) use action::Action;
use anyhow::Result;
pub(crate) use execution_plan::ExecutionPlan;
use std::{ops::ControlFlow, time::Duration};

mod action;
mod execution_plan;

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
