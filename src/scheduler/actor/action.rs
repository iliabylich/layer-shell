use crate::{
    macros::fatal,
    scheduler::{timer::Timer, Actor},
    Command,
};
use anyhow::Result;
use std::{ops::ControlFlow, sync::mpsc::Receiver};

#[derive(Debug, Clone, Copy)]
pub(crate) enum Action {
    Tick,
    Exec,
}

impl Action {
    pub(crate) fn run(
        self,
        name: &'static str,
        module: &mut dyn Actor,
        tick_timer: &mut Option<Timer>,
        exec_timer: &mut Option<Timer>,
        rx: &Receiver<Command>,
    ) -> Result<()> {
        match self {
            Self::Tick => Self::tick(name, module, tick_timer)?,
            Self::Exec => Self::exec(name, module, exec_timer, rx),
        }

        Ok(())
    }

    fn tick(
        name: &'static str,
        module: &mut dyn Actor,
        maybe_timer: &mut Option<Timer>,
    ) -> Result<()> {
        let timer_before = *maybe_timer;

        if let Some(timer) = maybe_timer.as_mut() {
            if let ControlFlow::Continue(interval) = module.tick()? {
                timer.interval = interval;
                timer.tick();
            } else {
                log::info!("Disabling ticking for {name}");
                *maybe_timer = None;
            }
        } else {
            fatal!("Trying to run tick timer for module {name}, but it's disabled")
        }

        assert_ne!(
            timer_before,
            *maybe_timer,
            "expected timer to change; before: {:?}, after: {:?}",
            timer_before.map(|t| t.pretty()),
            maybe_timer.map(|t| t.pretty())
        );

        Ok(())
    }

    fn exec(
        name: &'static str,
        module: &mut dyn Actor,
        maybe_timer: &mut Option<Timer>,
        rx: &Receiver<Command>,
    ) {
        let timer_before = *maybe_timer;

        if let Some(timer) = maybe_timer.as_mut() {
            let mut should_stop = false;

            while let Ok(cmd) = rx.try_recv() {
                match module.exec(&cmd) {
                    Ok(ControlFlow::Continue(_)) => {}
                    Ok(ControlFlow::Break(())) => {
                        should_stop = true;

                        break;
                    }
                    Err(err) => log::error!(
                        "Module {}: failed to exec command {:?}: {:?}",
                        name,
                        cmd,
                        err
                    ),
                }
            }

            if should_stop {
                log::info!("Disabling processing of commands for {name}");

                *maybe_timer = None;
            } else {
                timer.tick();
            }
        } else {
            fatal!("Trying to exec exec timer for module {name}, but it's disabled");
        }

        assert_ne!(
            timer_before,
            *maybe_timer,
            "expected timer to change; before: {:?}, after: {:?}",
            timer_before.map(|t| t.pretty()),
            maybe_timer.map(|t| t.pretty())
        );
    }
}
