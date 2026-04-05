use crate::{
    dbus::{
        OutgoingMessage,
        decoder::{Body, IncomingMessage, MessageType},
    },
    sansio::{DBusConnectionKind, DBusQueue},
};
use anyhow::{Result, bail};
use core::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
enum OneshotState {
    None,
    WaitingForReply(u32),
    ReplyReceived,
}

pub(crate) struct MethodCall<In, Out, Data>
where
    In: 'static,
    Out: 'static,
    Data: Clone + Default + 'static,
{
    send: &'static dyn Fn(In, Data) -> OutgoingMessage,
    try_process: &'static dyn Fn(Body<'_>, Data) -> Result<Out>,
    kind: DBusConnectionKind,
    state: OneshotState,
    data: Option<Data>,
}

impl<In, Out, Data> MethodCall<In, Out, Data>
where
    Data: Clone + Default,
{
    pub(crate) fn with_data(self, data: Data) -> Self {
        Self {
            send: self.send,
            try_process: self.try_process,
            kind: self.kind,
            state: self.state,
            data: Some(data),
        }
    }

    pub(crate) fn send(&mut self, input: In) {
        if !matches!(self.state, OneshotState::None) {
            return;
        };

        let message: OutgoingMessage = (self.send)(input, self.data.clone().unwrap_or_default());
        let reply_serial = DBusQueue::push_back(self.kind, message);
        self.state = OneshotState::WaitingForReply(reply_serial);
    }

    pub(crate) fn try_recv(&mut self, message: IncomingMessage<'_>) -> Result<Option<Out>> {
        let OneshotState::WaitingForReply(reply_serial) = self.state else {
            return Ok(None);
        };
        if message.reply_serial != Some(reply_serial) {
            return Ok(None);
        }
        self.state = OneshotState::ReplyReceived;

        match message.message_type {
            MessageType::Error => {
                bail!("DBus error: {:?}", message.error_name)
            }
            MessageType::MethodReturn => {
                if let Some(body) = message.body {
                    Ok((self.try_process)(body, self.data.clone().unwrap_or_default()).ok())
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.state = OneshotState::None
    }

    pub(crate) const fn builder() -> OneshotMethodCallBuilder<In, Out, Data, NeedsSend> {
        OneshotMethodCallBuilder {
            send: &|_: In, _: Data| todo!(),
            try_process: &|_: Body<'_>, _: Data| todo!(),
            _state: PhantomData,
        }
    }
}

impl<In, Out, Data> core::fmt::Debug for MethodCall<In, Out, Data>
where
    Data: Clone + Default,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("OneshotMethodCall")
            .field("state", &self.state)
            .finish()
    }
}

pub(crate) struct NeedsSend;
pub(crate) struct NeedsTryProcess;
pub(crate) struct NeedsConnectionKind;

pub(crate) struct OneshotMethodCallBuilder<In, Out, Data, S>
where
    In: 'static,
    Out: 'static,
    Data: Default + Clone + 'static,
{
    send: &'static dyn Fn(In, Data) -> OutgoingMessage,
    try_process: &'static dyn Fn(Body<'_>, Data) -> Result<Out>,
    _state: PhantomData<S>,
}

impl<In, Out, Data> OneshotMethodCallBuilder<In, Out, Data, NeedsSend>
where
    Data: Default + Clone,
{
    pub(crate) const fn send(
        self,
        send: &'static dyn Fn(In, Data) -> OutgoingMessage,
    ) -> OneshotMethodCallBuilder<In, Out, Data, NeedsTryProcess> {
        OneshotMethodCallBuilder {
            send,
            try_process: &|_: Body<'_>, _: Data| todo!(),
            _state: PhantomData,
        }
    }
}
impl<In, Out, Data> OneshotMethodCallBuilder<In, Out, Data, NeedsTryProcess>
where
    Data: Default + Clone,
{
    pub(crate) const fn try_process(
        self,
        try_process: &'static dyn Fn(Body<'_>, Data) -> Result<Out>,
    ) -> OneshotMethodCallBuilder<In, Out, Data, NeedsConnectionKind> {
        OneshotMethodCallBuilder {
            send: self.send,
            try_process,
            _state: PhantomData,
        }
    }
}
impl<In, Out, Data> OneshotMethodCallBuilder<In, Out, Data, NeedsConnectionKind>
where
    Data: Default + Clone,
{
    pub(crate) const fn kind(self, kind: DBusConnectionKind) -> MethodCall<In, Out, Data> {
        MethodCall {
            send: self.send,
            try_process: self.try_process,
            kind,
            state: OneshotState::None,
            data: None,
        }
    }
}
