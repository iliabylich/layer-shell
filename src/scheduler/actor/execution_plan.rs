use crate::{
    macros::fatal,
    scheduler::{actor::Action, timer::Timer},
};
use anyhow::{bail, Result};
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub(crate) enum ExecutionPlan {
    Tick(Timer),
    ProcessIncomingCommands(Timer),
    Both {
        tick_at: Timer,
        process_incoming_commands_at: Timer,
    },
    None,
}

impl ExecutionPlan {
    pub(crate) fn initial() -> Self {
        Self::Both {
            tick_at: Timer::start_now(Duration::from_secs(1_000)),
            process_incoming_commands_at: Timer::start_now(Duration::from_millis(50)),
        }
    }

    pub(crate) fn first_timer_and_action(self) -> (Timer, Action) {
        match self {
            Self::Tick(timer) => (timer, Action::Tick),
            Self::ProcessIncomingCommands(timer) => (timer, Action::ProcessIncomingCommands),
            Self::Both {
                tick_at,
                process_incoming_commands_at,
            } => {
                if tick_at < process_incoming_commands_at {
                    (tick_at, Action::Tick)
                } else {
                    (
                        process_incoming_commands_at,
                        Action::ProcessIncomingCommands,
                    )
                }
            }
            Self::None => fatal!("Execution plan has been disabled, can't re-enable"),
        }
    }

    pub(crate) fn first_timer(self) -> Timer {
        let (timer, _) = self.first_timer_and_action();
        timer
    }

    pub(crate) fn tick_timer(&mut self) -> Result<&mut Timer> {
        match self {
            ExecutionPlan::Tick(timer) => Ok(timer),
            ExecutionPlan::Both { tick_at, .. } => Ok(tick_at),
            plan => bail!("Execution plan {:?} doesn't have tick timer", plan),
        }
    }

    pub(crate) fn commands_timer(&mut self) -> Result<&mut Timer> {
        match self {
            ExecutionPlan::ProcessIncomingCommands(timer) => Ok(timer),
            ExecutionPlan::Both {
                process_incoming_commands_at,
                ..
            } => Ok(process_incoming_commands_at),
            plan => bail!("Execution plan {:?} doesn't have commands timer", plan),
        }
    }

    pub(crate) fn update_ticking_interval(&mut self, interval: Duration) {
        match self {
            Self::Tick(timer) => timer.interval = interval,
            Self::Both { tick_at, .. } => tick_at.interval = interval,
            Self::ProcessIncomingCommands(_) | Self::None => fatal!("ticking is disabled"),
        }
    }

    pub(crate) fn disable_ticking(&mut self) {
        match self {
            Self::Tick(_) => *self = Self::None,
            Self::Both {
                process_incoming_commands_at,
                ..
            } => *self = Self::ProcessIncomingCommands(*process_incoming_commands_at),
            Self::ProcessIncomingCommands(_) | Self::None => fatal!("ticking is ALREADY disabled"),
        }
    }

    pub(crate) fn disable_processing_of_commands(&mut self) {
        match self {
            Self::ProcessIncomingCommands(_) => *self = Self::None,
            Self::Both { tick_at, .. } => *self = Self::Tick(*tick_at),
            Self::Tick(_) | Self::None => fatal!("processing of commands is ALREADY disabled"),
        }
    }
}

impl Ord for ExecutionPlan {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.first_timer().cmp(&other.first_timer())
    }
}

impl PartialOrd for ExecutionPlan {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ExecutionPlan {
    fn eq(&self, other: &Self) -> bool {
        self.first_timer() == other.first_timer()
    }
}

impl Eq for ExecutionPlan {}

impl ExecutionPlan {
    pub(crate) fn checksum(&self) -> [u64; 5] {
        match self {
            ExecutionPlan::Tick(timer) => [1, timer.ts, timer.interval.as_millis() as u64, 0, 0],
            ExecutionPlan::ProcessIncomingCommands(timer) => {
                [2, timer.ts, timer.interval.as_millis() as u64, 0, 0]
            }
            ExecutionPlan::Both {
                tick_at,
                process_incoming_commands_at,
            } => [
                3,
                tick_at.ts,
                tick_at.interval.as_millis() as u64,
                process_incoming_commands_at.ts,
                process_incoming_commands_at.interval.as_millis() as u64,
            ],
            ExecutionPlan::None => [4, 0, 0, 0, 0],
        }
    }
}
