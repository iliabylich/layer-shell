use crate::utils::{
    StringRef, StringRefExt as _,
    dbus::{infallible_property::InfalliblePropertyGetAndSubscribe, queue::DBusQueue},
};
use dbus::{
    IncomingMessage,
    messages::network_manager::{SSID, Strength},
};

pub(crate) struct SsidAndStrength {
    ssid: InfalliblePropertyGetAndSubscribe<SSID<100, StringRef>>,
    strength: InfalliblePropertyGetAndSubscribe<Strength<StringRef>>,
}

#[derive(Debug)]
pub(crate) struct SsidAndStrengthEvent {
    pub(crate) ssid: Option<StringRef>,
    pub(crate) strength: Option<u8>,
}

impl SsidAndStrength {
    pub(crate) const fn new() -> Self {
        Self {
            ssid: InfalliblePropertyGetAndSubscribe::new(),
            strength: InfalliblePropertyGetAndSubscribe::new(),
        }
    }

    pub(crate) fn start(&mut self, path: StringRef, q: &mut DBusQueue) {
        self.ssid.get_and_subscribe(SSID::new(path.clone()), q);
        self.strength.get_and_subscribe(Strength::new(path), q);
    }

    pub(crate) fn stop(&mut self, q: &mut DBusQueue) {
        self.ssid.unsubscribe(q);
        self.strength.unsubscribe(q);
    }

    pub(crate) fn handle(
        &mut self,
        message: IncomingMessage<'_>,
        q: &mut DBusQueue,
    ) -> Option<SsidAndStrengthEvent> {
        let mut e = SsidAndStrengthEvent {
            ssid: None,
            strength: None,
        };

        if let Some((buf, len)) = self.ssid.handle_reply_or_signal(message, q) {
            let Some(buf) = buf.get(..len) else {
                log::error!("SSID property returned malformed data");
                return None;
            };
            let ssid = String::from_utf8_lossy(buf).to_string();
            e.ssid = Some(StringRef::new(&ssid));
        }

        if let Some(strength) = self.strength.handle_reply_or_signal(message, q) {
            e.strength = Some(strength);
        }

        if e.ssid.is_some() || e.strength.is_some() {
            Some(e)
        } else {
            None
        }
    }
}
