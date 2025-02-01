use std::time::Duration;

use crate::{
    scheduler::{Module, RepeatingModule},
    Command,
};
use anyhow::Result;
use state::State;

mod state;
mod system_app;

pub(crate) struct AppList {
    state: State,
}

impl Module for AppList {
    const NAME: &str = "AppList";

    fn start() -> Result<Option<Box<dyn RepeatingModule>>> {
        Ok(Some(Box::new(Self {
            state: State::new(),
        })))
    }
}

impl RepeatingModule for AppList {
    fn tick(&mut self) -> Result<Duration> {
        Ok(Duration::from_secs(100_000))
    }

    fn exec(&mut self, cmd: &Command) -> Result<()> {
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
        Ok(())
    }
}
