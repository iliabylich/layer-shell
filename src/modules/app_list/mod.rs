use crate::{scheduler::Actor, Command};
use anyhow::Result;
use state::State;
use std::{ops::ControlFlow, time::Duration};

mod state;
mod system_app;

#[derive(Debug)]
pub(crate) struct AppList {
    state: State,
}

impl Actor for AppList {
    fn name() -> &'static str {
        "AppList"
    }

    fn start() -> Result<Box<dyn Actor>> {
        Ok(Box::new(Self {
            state: State::new(),
        }))
    }

    fn tick(&mut self) -> Result<ControlFlow<(), Duration>> {
        Ok(ControlFlow::Break(()))
    }

    fn exec(&mut self, cmd: &Command) -> Result<ControlFlow<()>> {
        match cmd {
            Command::AppListReset => self.state.reset()?,
            Command::AppListGoUp => self.state.go_up()?,
            Command::AppListGoDown => self.state.go_down()?,
            Command::AppListSetSearch { search } => {
                let search = String::from(search.clone());
                self.state.set_search(search.to_string())?;
            }
            Command::AppListExecSelected => self.state.exec_selected()?,

            _ => {}
        }
        Ok(ControlFlow::Continue(()))
    }
}
