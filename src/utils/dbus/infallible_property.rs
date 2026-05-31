use crate::utils::dbus::queue::DBusQueue;
use dbus::{
    DBusError, IncomingMessage,
    messaging::property::{Property, PropertyGetAndSubscribe},
};

pub(crate) struct InfalliblePropertyGetAndSubscribe<T>
where
    T: Property,
{
    inner: Option<PropertyGetAndSubscribe<T>>,
    q: &'static mut DBusQueue,
}

impl<T> InfalliblePropertyGetAndSubscribe<T>
where
    T: Property,
{
    pub(crate) const fn new(q: &'static mut DBusQueue) -> Self {
        Self { inner: None, q }
    }

    fn try_get_and_subscribe(&mut self, property: T) -> Result<(), DBusError> {
        self.inner = Some(PropertyGetAndSubscribe::get_and_subscribe(
            property,
            &mut [0; 1_024],
            self.q,
        )?);
        Ok(())
    }

    fn try_get(&mut self, property: T) -> Result<(), DBusError> {
        self.inner = Some(PropertyGetAndSubscribe::get(
            property,
            &mut [0; 1_024],
            self.q,
        )?);
        Ok(())
    }

    pub(crate) fn get_and_subscribe(&mut self, property: T) {
        if let Err(err) = self.try_get_and_subscribe(property) {
            log::error!("{err:?}");
            self.unsubscribe();
        }
    }

    pub(crate) fn get(&mut self, property: T) {
        if let Err(err) = self.try_get(property) {
            log::error!("{err:?}");
            self.unsubscribe();
        }
    }

    pub(crate) fn subscribe(&mut self) {
        let Some(inner) = self.inner.as_ref() else {
            return;
        };
        if let Err(err) = inner.subscribe(&mut [0; 1_024], self.q) {
            log::error!("{err:?}");
        }
    }

    pub(crate) fn unsubscribe(&mut self) {
        let Some(inner) = self.inner.take() else {
            return;
        };
        if let Err(err) = inner.unsubscribe(&mut [0; 1_024], self.q) {
            log::error!("{err:?}");
        }
    }

    pub(crate) fn handle_reply_or_signal<'a>(
        &mut self,
        message: IncomingMessage<'a>,
    ) -> Option<T::Output<'a>> {
        match self.inner.as_ref()?.handle_reply_or_signal(message) {
            Ok(Some(out)) => Some(out),
            Ok(None) => None,
            Err(err) => {
                log::error!("{err:?}");
                self.unsubscribe();
                None
            }
        }
    }
}
