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

#[derive(Debug, Clone, Copy)]
enum State {
    None,
    Volume(u32),
    Muted(bool),
    Both(u32, bool),
}
impl State {
    fn offer(&mut self, volume: Option<u32>, muted: Option<bool>) -> bool {
        match (*self, volume, muted) {
            (_, None, None) => {}

            (Self::Muted(_) | Self::None, None, Some(muted)) => *self = Self::Muted(muted),

            (Self::Volume(_) | Self::None, Some(volume), None) => *self = Self::Volume(volume),

            (Self::Muted(_) | Self::Volume(_) | Self::None, Some(volume), Some(muted))
            | (Self::Muted(muted), Some(volume), None)
            | (Self::Volume(volume), None, Some(muted)) => {
                *self = Self::Both(volume, muted);
                EventQueue::push_back(Event::InitialSound { volume, muted });
                return true;
            }

            (Self::Both(volume, _), None, Some(muted)) => {
                *self = Self::Both(volume, muted);
                EventQueue::push_back(Event::MuteChanged { muted });
            }
            (Self::Both(_, muted), Some(volume), None) => {
                *self = Self::Both(volume, muted);
                EventQueue::push_back(Event::VolumeChanged { volume });
            }
            (Self::Both(_, _), Some(volume), Some(muted)) => {
                *self = Self::Both(volume, muted);
                EventQueue::push_back(Event::VolumeChanged { volume });
                EventQueue::push_back(Event::MuteChanged { muted });
            }
        }

        false
    }
}

impl Sound {
    pub(crate) const fn new() -> Self {
        Self {
            volume: InfalliblePropertyGetAndSubscribe::new(SessionDBus::queue()),
            muted: InfalliblePropertyGetAndSubscribe::new(SessionDBus::queue()),
            state: State::None,
            healthy: true,
        }
    }

    pub(crate) fn start(&mut self) {
        self.volume.get(Volume);
        self.muted.get(Muted);
        self.state = State::None;
    }

    pub(crate) fn handle(&mut self, message: IncomingMessage<'_>) {
        let volume = self.volume.handle_reply_or_signal(message);
        let muted = self.muted.handle_reply_or_signal(message);

        let got_both = self.state.offer(volume, muted);
        if got_both {
            self.volume.subscribe();
            self.muted.subscribe();
        }
    }

    pub(crate) fn tick(&mut self, tick: u64) {
        if !self.healthy && tick.is_multiple_of(2) {
            self.healthy = true;
            self.start();
        }
    }
}
