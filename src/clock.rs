use crate::{
    Event,
    liburing::{IoUringActor, IoUring},
    timerfd::Tick,
    user_data::UserData,
};
use anyhow::Result;

enum State {
    CanTick,
    WaitingForTimer,
}

pub(crate) struct Clock {
    state: State,
}

impl Clock {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            state: State::WaitingForTimer,
        })
    }
}

impl IoUringActor for Clock {
    fn drain_once(&mut self, _ring: &mut IoUring, events: &mut Vec<Event>) -> Result<bool> {
        match self.state {
            State::CanTick => {
                let time = chrono::Local::now()
                    .format("%H:%M:%S | %b %e | %a")
                    .to_string();
                events.push(Event::Clock { time: time.into() });
                self.state = State::WaitingForTimer;
            }
            State::WaitingForTimer => {}
        }

        Ok(false)
    }

    fn feed(
        &mut self,
        _ring: &mut IoUring,
        _user_data: UserData,
        _res: i32,
        _events: &mut Vec<Event>,
    ) -> Result<()> {
        Ok(())
    }

    fn on_tick(&mut self, tick: Tick) -> Result<()> {
        if tick.is_multiple_of(1) {
            self.state = State::CanTick
        }
        Ok(())
    }
}
