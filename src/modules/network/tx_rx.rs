use crate::dbus::{
    DBus, Message,
    messages::{
        interface_is,
        org_freedesktop_dbus::{AddMatch, PropertiesChanged, SetProperty},
        path_is, value_is,
    },
    types::Value,
};
use anyhow::{Context, Result, bail};

pub(crate) struct TxRx {
    path: Option<String>,
}

#[derive(Debug)]
pub(crate) struct TxRxEvent {
    pub(crate) tx: Option<u64>,
    pub(crate) rx: Option<u64>,
}

impl TxRx {
    pub(crate) fn new() -> Self {
        Self { path: None }
    }

    fn configure(&self, dbus: &mut DBus, path: &str) {
        let mut message: Message = SetProperty::new(
            "org.freedesktop.NetworkManager",
            path,
            "org.freedesktop.NetworkManager.Device.Statistics",
            "RefreshRateMs",
            Value::UInt32(1000),
        )
        .into();
        dbus.enqueue(&mut message);
    }

    fn subscribe(&mut self, dbus: &mut DBus, path: &str) {
        let mut message: Message = AddMatch::new(path).into();
        dbus.enqueue(&mut message);
        self.path = Some(path.to_string())
    }

    fn unsubscribe(&mut self, dbus: &mut DBus) {
        let Some(path) = self.path.take() else {
            return;
        };
        let mut message: Message = AddMatch::new(&path).into();
        dbus.enqueue(&mut message);
    }

    pub(crate) fn reset(&mut self, dbus: &mut DBus) {
        self.unsubscribe(dbus);
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus, path: &str) {
        self.unsubscribe(dbus);
        self.configure(dbus, path);
        self.subscribe(dbus, path);
    }

    fn try_parse_signal(&self, message: &Message) -> Result<TxRxEvent> {
        let expected_path = self.path.as_ref().context("not subscribed")?.as_str();

        let PropertiesChanged {
            path,
            changes,
            interface,
        } = PropertiesChanged::try_from(message)?;

        path_is!(path, expected_path);
        interface_is!(
            interface,
            "org.freedesktop.NetworkManager.Device.Statistics"
        );

        let tx = if let Some(tx) = changes.get("TxBytes") {
            value_is!(tx, Value::UInt64(tx));
            Some(*tx)
        } else {
            None
        };

        let rx = if let Some(rx) = changes.get("RxBytes") {
            value_is!(rx, Value::UInt64(rx));
            Some(*rx)
        } else {
            None
        };

        if tx.is_some() || rx.is_some() {
            Ok(TxRxEvent { tx, rx })
        } else {
            bail!("unrelated")
        }
    }

    pub(crate) fn on_message(&self, message: &Message) -> Option<TxRxEvent> {
        if let Ok(e) = self.try_parse_signal(message) {
            return Some(e);
        }

        None
    }
}
