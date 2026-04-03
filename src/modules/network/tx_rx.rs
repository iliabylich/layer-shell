use crate::{
    dbus::{
        MethodCall, Subscription,
        decoder::{IncomingMessage, Value},
        messages::{interface_is, org_freedesktop_dbus::SetProperty, path_is, value_is},
    },
    ffi::ShortString,
    sansio::DBusConnectionKind,
};
use anyhow::Context as _;

pub(crate) struct TxRx {
    oneshot: MethodCall<ShortString, (), ()>,
    subscription: Subscription<TxRxEvent>,
}

#[derive(Debug)]
pub(crate) struct TxRxEvent {
    pub(crate) tx: Option<u64>,
    pub(crate) rx: Option<u64>,
}

impl TxRx {
    pub(crate) fn new() -> Self {
        Self {
            oneshot: CONFIGURE,
            subscription: SUBSCRIPTION,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.oneshot.reset();
        self.subscription.reset();
    }

    pub(crate) fn init(&mut self, path: ShortString) {
        self.oneshot.send(path);
        self.subscription.start(
            ShortString::new_const("org.freedesktop.NetworkManager"),
            path,
        );
    }

    pub(crate) fn on_message(&self, message: IncomingMessage<'_>) -> Option<TxRxEvent> {
        self.subscription.process(message)
    }
}

const CONFIGURE: MethodCall<ShortString, (), ()> = MethodCall::builder()
    .send(&|path, _data| {
        use crate::dbus::types::Value;

        SetProperty::new(
            ShortString::new_const("org.freedesktop.NetworkManager"),
            path,
            ShortString::new_const("org.freedesktop.NetworkManager.Device.Statistics"),
            ShortString::new_const("RefreshRateMs"),
            Value::UInt32(1000),
        )
        .into()
    })
    .try_process(&|_body, _data| unreachable!())
    .kind(DBusConnectionKind::System);

const SUBSCRIPTION: Subscription<TxRxEvent> = Subscription::builder()
    .try_process(&|mut body, path, subscribed_to| {
        path_is!(path, subscribed_to);

        let interface = body.try_next()?.context("no Interface in Body")?;
        value_is!(interface, Value::String(interface));
        interface_is!(
            interface,
            "org.freedesktop.NetworkManager.Device.Statistics"
        );

        let attributes = body.try_next()?.context("no Attributes in Body")?;
        value_is!(attributes, Value::Array(attributes));
        let mut iter = attributes.iter();
        let mut tx = None;
        let mut rx = None;
        while let Some(attribute) = iter.try_next()? {
            value_is!(attribute, Value::DictEntry(attribute));
            let (key, value) = attribute.key_value()?;
            value_is!(key, Value::String(key));
            value_is!(value, Value::Variant(value));

            if key == "TxBytes" {
                let value = value.materialize()?;
                value_is!(value, Value::UInt64(value));
                tx = Some(value);
            } else if key == "RxBytes" {
                let value = value.materialize()?;
                value_is!(value, Value::UInt64(value));
                rx = Some(value);
            }
        }

        Ok(TxRxEvent { tx, rx })
    })
    .kind(DBusConnectionKind::System);
