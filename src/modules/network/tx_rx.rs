use crate::utils::{
    StringRef,
    dbus::{infallible_property::InfalliblePropertyGetAndSubscribe, queue::DBusQueue},
};
use dbus::{
    DBusError, IncomingMessage,
    messages::network_manager::{RefreshRateMs, RxBytes, TxBytes},
};

pub(crate) struct TxRx {
    tx: InfalliblePropertyGetAndSubscribe<TxBytes<StringRef>>,
    rx: InfalliblePropertyGetAndSubscribe<RxBytes<StringRef>>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TxRxEvent {
    pub(crate) tx: Option<u64>,
    pub(crate) rx: Option<u64>,
}

impl TxRx {
    pub(crate) const fn new() -> Self {
        Self {
            tx: InfalliblePropertyGetAndSubscribe::new(),
            rx: InfalliblePropertyGetAndSubscribe::new(),
        }
    }

    pub(crate) fn start(&mut self, path: StringRef, q: &mut DBusQueue) {
        if let Err(err) = Configure::send(path.as_str(), q) {
            log::error!("{err:?}");
            return;
        }
        self.tx.get_and_subscribe(TxBytes::new(path.clone()), q);
        self.rx.get_and_subscribe(RxBytes::new(path), q);
    }

    pub(crate) fn stop(&mut self, q: &mut DBusQueue) {
        self.tx.unsubscribe(q);
        self.rx.unsubscribe(q);
    }

    pub(crate) fn handle(
        &mut self,
        message: IncomingMessage<'_>,
        q: &mut DBusQueue,
    ) -> Option<TxRxEvent> {
        let mut e = TxRxEvent { tx: None, rx: None };

        if let Some(tx) = self.tx.handle_reply_or_signal(message, q) {
            e.tx = Some(tx);
        }

        if let Some(rx) = self.rx.handle_reply_or_signal(message, q) {
            e.rx = Some(rx);
        }

        if e.tx.is_some() || e.rx.is_some() {
            Some(e)
        } else {
            None
        }
    }
}

struct Configure;
impl Configure {
    fn send(path: &str, q: &mut DBusQueue) -> Result<(), DBusError> {
        let mut buf = [0; 1_024];
        let encoded = RefreshRateMs::encode_set_property(&mut buf, path, 1_000)?;
        q.push_raw(encoded);
        Ok(())
    }
}
