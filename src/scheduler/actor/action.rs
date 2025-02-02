use crate::{
    macros::fatal,
    scheduler::{actor::ExecutionPlan, Actor},
    Command,
};
use anyhow::{Context, Result};
use std::{ops::ControlFlow, sync::mpsc::Receiver};

#[derive(Debug, Clone, Copy)]
pub(crate) enum Action {
    Tick,
    Exec,
}

impl Action {
    pub(crate) fn exec(
        self,
        name: &'static str,
        module: &mut dyn Actor,
        execution_plan: &mut ExecutionPlan,
        rx: &Receiver<Command>,
    ) -> Result<()> {
        let hash_before = execution_plan.checksum();

        match self {
            Self::Tick => {
                if let ControlFlow::Continue(interval) = module.tick()? {
                    execution_plan.update_ticking_interval(interval);

                    execution_plan
                        .tick_timer()
                        .with_context(|| format!("failed to find tick timer for module {name}"))?
                        .tick();
                } else {
                    log::info!("Disabling ticking for {name}");
                    execution_plan.disable_ticking();
                }
            }
            Self::Exec => {
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

                    execution_plan.disable_processing_of_commands();
                } else {
                    execution_plan
                        .commands_timer()
                        .with_context(|| {
                            format!("failed to find commands timer for module {name}")
                        })?
                        .tick();
                }
            }
        }

        if execution_plan.checksum() == hash_before {
            fatal!(
                "[{} {:?}] bug: no changes in the execution plan, infinite loop? {:?}",
                name,
                self,
                execution_plan
            );
        }

        Ok(())
    }
}
