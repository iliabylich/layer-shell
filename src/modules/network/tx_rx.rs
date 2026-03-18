use crate::{
    dbus::{
        Message, Oneshot, OneshotResource, Subscription, SubscriptionResource,
        decoder::{Body, IncomingMessage, Value},
        messages::{interface_is, org_freedesktop_dbus::SetProperty, path_is, value_is},
    },
    sansio::DBusQueue,
};
use anyhow::{Context, Result};

pub(crate) struct TxRx {
    oneshot: Oneshot<Resource>,
    subscription: Subscription<Resource>,
}

#[derive(Debug)]
pub(crate) struct TxRxEvent {
    pub(crate) tx: Option<u64>,
    pub(crate) rx: Option<u64>,
}

impl TxRx {
    pub(crate) fn new(queue: DBusQueue) -> Self {
        Self {
            oneshot: Oneshot::new(Resource::default(), queue.copy()),
            subscription: Subscription::new(Resource::default(), queue.copy()),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.oneshot.reset();
        self.subscription.reset();
    }

    pub(crate) fn init(&mut self, path: &str) {
        self.oneshot.start(path.to_string());
        self.subscription
            .start("org.freedesktop.NetworkManager", path);
    }

    pub(crate) fn on_message(&self, message: IncomingMessage<'_>) -> Option<TxRxEvent> {
        self.subscription.process(message)
    }
}

#[derive(Default)]
struct Resource {
    path: Option<String>,
}

impl OneshotResource for Resource {
    type Input = String;
    type Output = ();

    fn make_request(&self, path: String) -> Message<'static> {
        use crate::dbus::types::Value;

        SetProperty::new(
            "org.freedesktop.NetworkManager",
            path,
            "org.freedesktop.NetworkManager.Device.Statistics",
            "RefreshRateMs",
            Value::UInt32(1000),
        )
        .into()
    }

    fn try_process(&self, _body: Body<'_>) -> Result<Self::Output> {
        panic!("doesn't have to be checked")
    }
}

impl SubscriptionResource for Resource {
    type Output = TxRxEvent;

    fn try_process(&self, path: &str, mut body: Body<'_>) -> Result<Self::Output> {
        path_is!(path, self.path.as_deref().context("no path")?);

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
    }

    fn set_path(&mut self, path: String) {
        self.path = Some(path)
    }
}
