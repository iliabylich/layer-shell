use crate::{
    Event, event_queue::EventQueue, modules::SessionDBus,
    utils::dbus::infallible_property::InfalliblePropertyGetAndSubscribe,
};
use dbus::IncomingMessage;
use properties::{Muted, Volume};

mod properties;

pub(crate) struct Sound {
    volume: InfalliblePropertyGetAndSubscribe<Volume>,
    muted: InfalliblePropertyGetAndSubscribe<Muted>,
    state: State,
    healthy: bool,
}

enum State {
    Partial(Option<u32>, Option<bool>),
    Full(u32, bool),
}
impl State {
    const fn new() -> Self {
        Self::Partial(None, None)
    }
    fn got(&mut self, new_volume: Option<u32>, new_muted: Option<bool>) {
        match self {
            Self::Partial(prev_volume, prev_muted) => {
                let volume = new_volume.or(*prev_volume);
                let muted = new_muted.or(*prev_muted);
                if let Some(volume) = volume
                    && let Some(muted) = muted
                {
                    *self = Self::Full(volume, muted);
                } else {
                    *self = Self::Partial(volume, muted);
                }
            }
            Self::Full(volume, muted) => {
                let mut changed = false;
                if let Some(new_volume) = new_volume
                    && new_volume != *volume
                {
                    *volume = new_volume;
                    changed = true;
                }
                if let Some(new_muted) = new_muted
                    && new_muted != *muted
                {
                    *muted = new_muted;
                    changed = true;
                }
                if changed {
                    EventQueue::push_back(Event::Sound {
                        volume: *volume,
                        muted: *muted,
                    });
                }
            }
        }
    }
}

impl Sound {
    pub(crate) const fn new() -> Self {
        Self {
            volume: InfalliblePropertyGetAndSubscribe::new(SessionDBus::queue()),
            muted: InfalliblePropertyGetAndSubscribe::new(SessionDBus::queue()),
            state: State::new(),
            healthy: true,
        }
    }

    pub(crate) fn start(&mut self) {
        self.volume.get_and_subscribe(Volume);
        self.muted.get_and_subscribe(Muted);
    }

    pub(crate) fn handle(&mut self, message: IncomingMessage<'_>) {
        let volume = self.volume.handle_reply_or_signal(message);
        let muted = self.muted.handle_reply_or_signal(message);

        self.state.got(volume, muted);
    }

    pub(crate) fn tick(&mut self, tick: u64) {
        if !self.healthy && tick.is_multiple_of(2) {
            self.healthy = true;
            self.start();
        }
    }
}
