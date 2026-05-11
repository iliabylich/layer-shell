use crate::{modules::SystemDBus, utils::StringRef};
use anyhow::Context as _;
use mini_sansio_dbus::{
    IncomingMessage, IncomingValue, IncompleteMethodCall, MethodCall, OutgoingValue, Subscription,
    interface_is, messages::org_freedesktop_dbus::SetProperty, path_is, value_is,
};

pub(crate) struct TxRx {
    oneshot: MethodCall<StringRef, (), ()>,
    subscription: Subscription<TxRxEvent>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TxRxEvent {
    pub(crate) tx: Option<u64>,
    pub(crate) rx: Option<u64>,
}

impl TxRx {
    pub(crate) const fn new() -> Self {
        Self {
            oneshot: CONFIGURE.with_data(()),
            subscription: SUBSCRIPTION,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.oneshot.reset();
        self.subscription.reset(SystemDBus::queue());
    }

    pub(crate) fn init(&mut self, path: StringRef) {
        self.subscription.start(
            "org.freedesktop.NetworkManager",
            path.to_string(),
            SystemDBus::queue(),
        );
        self.oneshot.send(path, SystemDBus::queue());
    }

    pub(crate) fn on_message(&self, message: IncomingMessage<'_>) -> Option<TxRxEvent> {
        self.subscription.process(message)
    }
}

const CONFIGURE: IncompleteMethodCall<StringRef, (), ()> =
    MethodCall::new(&|path: StringRef, _data| {
        SetProperty::build(
            "org.freedesktop.NetworkManager",
            path.as_str(),
            "org.freedesktop.NetworkManager.Device.Statistics",
            "RefreshRateMs",
            OutgoingValue::UInt32(1000),
        )
    })
    .try_process(&|_body, _data| Err(anyhow::anyhow!("not meant to be called").into()));

const SUBSCRIPTION: Subscription<TxRxEvent> =
    Subscription::new(&|mut body, path, subscribed_to| {
        path_is!(path, subscribed_to);

        let interface = body.try_next()?.context("no Interface in Body")?;
        value_is!(interface, IncomingValue::String(interface));
        interface_is!(
            interface,
            "org.freedesktop.NetworkManager.Device.Statistics"
        );

        let attributes = body.try_next()?.context("no Attributes in Body")?;
        value_is!(attributes, IncomingValue::Array(attributes));
        let mut iter = attributes.items_iter();
        let mut tx = None;
        let mut rx = None;
        while let Some(attribute) = iter.try_next()? {
            value_is!(attribute, IncomingValue::DictEntry(attribute));
            let (key, value) = attribute.key_value()?;
            value_is!(key, IncomingValue::String(key));
            value_is!(value, IncomingValue::Variant(value));

            if key == "TxBytes" {
                let value = value.materialize()?;
                value_is!(value, IncomingValue::UInt64(value));
                tx = Some(value);
            } else if key == "RxBytes" {
                let value = value.materialize()?;
                value_is!(value, IncomingValue::UInt64(value));
                rx = Some(value);
            }
        }

        Ok(TxRxEvent { tx, rx })
    });
