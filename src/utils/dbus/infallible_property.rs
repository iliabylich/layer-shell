use dbus::{
    DBusError, IncomingMessage, OutgoingQueue,
    messaging::{property::Property, reply_handler::ReplyHandler},
};

pub(crate) struct InfalliblePropertyGetAndSubscribe<T>
where
    T: Property,
{
    property: Option<T>,
    reply_handler: Option<ReplyHandler<T>>,
}

impl<T> InfalliblePropertyGetAndSubscribe<T>
where
    T: Property,
{
    pub(crate) const fn new() -> Self {
        Self {
            property: None,
            reply_handler: None,
        }
    }

    fn try_get_and_subscribe(
        &mut self,
        property: T,
        q: &mut impl OutgoingQueue,
    ) -> Result<(), DBusError> {
        let mut bytes = [0; 1_024];

        let buf = property.encode_get(&mut bytes)?;
        let serial = q.push_raw(buf);
        let reply_handler = ReplyHandler::new(serial, property.clone());

        let buf = property.encode_subscribe(&mut bytes)?;
        q.push_raw(buf);

        self.property = Some(property);
        self.reply_handler = Some(reply_handler);
        Ok(())
    }

    fn try_get(&mut self, property: T, q: &mut impl OutgoingQueue) -> Result<(), DBusError> {
        let mut bytes = [0; 1_024];

        let buf = property.encode_get(&mut bytes)?;
        let serial = q.push_raw(buf);
        let reply_handler = ReplyHandler::new(serial, property.clone());

        self.property = Some(property);
        self.reply_handler = Some(reply_handler);
        Ok(())
    }

    pub(crate) fn get_and_subscribe(&mut self, property: T, q: &mut impl OutgoingQueue) {
        if let Err(err) = self.try_get_and_subscribe(property, q) {
            log::error!("{err:?}");
            self.unsubscribe(q);
        }
    }

    pub(crate) fn get(&mut self, property: T, q: &mut impl OutgoingQueue) {
        if let Err(err) = self.try_get(property, q) {
            log::error!("{err:?}");
            self.unsubscribe(q);
        }
    }

    #[expect(dead_code)]
    pub(crate) fn subscribe(&self, q: &mut impl OutgoingQueue) {
        let Some(property) = self.property.as_ref() else {
            return;
        };
        let mut buf = [0; 1_024];
        match property.encode_subscribe(&mut buf) {
            Ok(buf) => {
                q.push_raw(buf);
            }
            Err(err) => {
                log::error!("{err:?}");
            }
        }
    }

    pub(crate) fn unsubscribe(&mut self, q: &mut impl OutgoingQueue) {
        let Some(property) = self.property.take() else {
            return;
        };
        let mut buf = [0; 1_024];
        match property.encode_unsubscribe(&mut buf) {
            Ok(buf) => {
                q.push_raw(buf);
            }
            Err(err) => {
                log::error!("{err:?}");
            }
        }
    }

    fn try_handle_reply_or_signal<'a>(
        &self,
        message: IncomingMessage<'a>,
    ) -> Result<Option<T::Output<'a>>, DBusError> {
        let Some(reply_handler) = self.reply_handler.as_ref() else {
            return Ok(None);
        };
        let Some(property) = self.property.as_ref() else {
            return Ok(None);
        };

        if let Some(out) = reply_handler.handle(message)? {
            Ok(Some(out))
        } else if let Some(out) = property.handle_signal(message)? {
            Ok(Some(out))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn handle_reply_or_signal<'a>(
        &mut self,
        message: IncomingMessage<'a>,
        q: &mut impl OutgoingQueue,
    ) -> Option<T::Output<'a>> {
        match self.try_handle_reply_or_signal(message) {
            Ok(Some(out)) => Some(out),
            Ok(None) => None,
            Err(err) => {
                log::error!("{err:?}");
                self.unsubscribe(q);
                None
            }
        }
    }
}
