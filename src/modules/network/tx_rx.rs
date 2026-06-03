use crate::{
    modules::SystemDBus,
    utils::{StringRef, dbus::infallible_property::InfalliblePropertyGetAndSubscribe},
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
            tx: InfalliblePropertyGetAndSubscribe::new(SystemDBus::queue()),
            rx: InfalliblePropertyGetAndSubscribe::new(SystemDBus::queue()),
        }
    }

    pub(crate) fn start(&mut self, path: StringRef) {
        if let Err(err) = Configure::send(path.as_str()) {
            log::error!("{err:?}");
            return;
        }
        self.tx.get_and_subscribe(TxBytes::new(path.clone()));
        self.rx.get_and_subscribe(RxBytes::new(path));
    }

    pub(crate) fn stop(&mut self) {
        self.tx.unsubscribe();
        self.rx.unsubscribe();
    }

    pub(crate) fn handle(&mut self, message: IncomingMessage<'_>) -> Option<TxRxEvent> {
        let mut e = TxRxEvent { tx: None, rx: None };

        if let Some(tx) = self.tx.handle_reply_or_signal(message) {
            e.tx = Some(tx);
        }

        if let Some(rx) = self.rx.handle_reply_or_signal(message) {
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
    fn send(path: &str) -> Result<(), DBusError> {
        let mut buf = [0; 1_024];
        let encoded = RefreshRateMs::encode_set_property(&mut buf, path, 1_000)?;
        SystemDBus::queue().push_raw(encoded);
        Ok(())
    }
}
