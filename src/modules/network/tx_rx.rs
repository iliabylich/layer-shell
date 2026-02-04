use crate::{
    dbus::{
        DBus, Message, Oneshot, OneshotResource, Subscription, SubscriptionResource,
        messages::{interface_is, org_freedesktop_dbus::SetProperty, path_is, value_is},
        types::Value,
    },
    liburing::IoUring,
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
    pub(crate) fn new() -> Self {
        Self {
            oneshot: Oneshot::new(Resource::default()),
            subscription: Subscription::new(Resource::default()),
        }
    }

    pub(crate) fn reset(&mut self, dbus: &mut DBus, ring: &mut IoUring) -> Result<()> {
        self.oneshot.reset();
        self.subscription.reset(dbus, ring)?;
        Ok(())
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus, path: &str, ring: &mut IoUring) -> Result<()> {
        self.oneshot.start(dbus, path.to_string(), ring)?;
        self.subscription.start(dbus, path, ring)?;
        Ok(())
    }

    pub(crate) fn on_message(&self, message: &Message) -> Option<TxRxEvent> {
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
        SetProperty::new(
            "org.freedesktop.NetworkManager",
            path,
            "org.freedesktop.NetworkManager.Device.Statistics",
            "RefreshRateMs",
            Value::UInt32(1000),
        )
        .into()
    }

    fn try_process(&self, _body: &[Value]) -> Result<Self::Output> {
        panic!("doesn't have to be checked")
    }
}

impl SubscriptionResource for Resource {
    type Output = TxRxEvent;

    fn try_process(&self, path: &str, interface: &str, items: &[Value]) -> Result<Self::Output> {
        interface_is!(
            interface,
            "org.freedesktop.NetworkManager.Device.Statistics"
        );
        path_is!(path, self.path.as_deref().context("no path")?);

        let mut tx = None;
        let mut rx = None;

        for item in items {
            value_is!(item, Value::DictEntry(key, value));
            value_is!(&**key, Value::String(key));
            value_is!(&**value, Value::Variant(value));

            if key == "TxBytes" {
                value_is!(&**value, Value::UInt64(value));
                tx = Some(*value);
            } else if key == "RxBytes" {
                value_is!(&**value, Value::UInt64(value));
                rx = Some(*value);
            }
        }

        Ok(TxRxEvent { tx, rx })
    }

    fn set_path(&mut self, path: String) {
        self.path = Some(path)
    }
}
