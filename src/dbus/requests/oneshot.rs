use crate::{
    dbus::{
        OutgoingMessage,
        decoder::{Body, IncomingMessage, MessageType},
    },
    sansio::{DBusConnectionKind, DBusQueue},
};
use anyhow::{Result, bail};

#[derive(Debug, Clone, Copy)]
enum OneshotState {
    None,
    WaitingForReply(u32),
    ReplyReceived,
}

pub(crate) struct OneshotMethodCall<In, Out, Data>
where
    In: 'static,
    Out: 'static,
    Data: Copy + Default + 'static,
{
    send: &'static dyn Fn(In, Data) -> OutgoingMessage,
    try_process: &'static dyn Fn(Body<'_>, Data) -> Result<Out>,
    kind: DBusConnectionKind,
    state: OneshotState,
    data: Option<Data>,
}

impl<In, Out, Data> OneshotMethodCall<In, Out, Data>
where
    Data: Copy + Default,
{
    pub(crate) const fn new_without_data(
        send: &'static dyn Fn(In, Data) -> OutgoingMessage,
        try_process: &'static dyn Fn(Body<'_>, Data) -> Result<Out>,
        kind: DBusConnectionKind,
    ) -> Self {
        Self {
            send,
            try_process,
            kind,
            state: OneshotState::None,
            data: None,
        }
    }

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

        let message: OutgoingMessage = (self.send)(input, self.data.unwrap_or_default());
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
                    Ok((self.try_process)(body, self.data.unwrap_or_default()).ok())
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
        OneshotMethodCallBuilder::<In, Out, Data, NeedsSend>::new()
    }
}

impl<In, Out, Data> std::fmt::Debug for OneshotMethodCall<In, Out, Data>
where
    Data: Copy + Default,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OneshotMethodCall")
            .field("state", &self.state)
            .finish()
    }
}

pub(crate) trait State {}
pub(crate) struct NeedsSend;
impl State for NeedsSend {}
pub(crate) struct NeedsTryProcess;
impl State for NeedsTryProcess {}
pub(crate) struct NeedsConnectionKind;
impl State for NeedsConnectionKind {}

pub(crate) struct OneshotMethodCallBuilder<In, Out, Data, S>
where
    In: 'static,
    Out: 'static,
    Data: Default + Copy + 'static,
    S: State,
{
    send: &'static dyn Fn(In, Data) -> OutgoingMessage,
    try_process: &'static dyn Fn(Body<'_>, Data) -> Result<Out>,
    _state: S,
}

impl<In, Out, Data> OneshotMethodCallBuilder<In, Out, Data, NeedsSend>
where
    Data: Default + Copy,
{
    const fn new() -> Self {
        Self {
            send: &|_: In, _: Data| todo!(),
            try_process: &|_: Body<'_>, _: Data| todo!(),
            _state: NeedsSend,
        }
    }

    pub(crate) const fn send(
        self,
        send: &'static dyn Fn(In, Data) -> OutgoingMessage,
    ) -> OneshotMethodCallBuilder<In, Out, Data, NeedsTryProcess> {
        OneshotMethodCallBuilder {
            send,
            try_process: &|_: Body<'_>, _: Data| todo!(),
            _state: NeedsTryProcess,
        }
    }
}
impl<In, Out, Data> OneshotMethodCallBuilder<In, Out, Data, NeedsTryProcess>
where
    Data: Default + Copy,
{
    pub(crate) const fn try_process(
        self,
        try_process: &'static dyn Fn(Body<'_>, Data) -> Result<Out>,
    ) -> OneshotMethodCallBuilder<In, Out, Data, NeedsConnectionKind> {
        OneshotMethodCallBuilder {
            send: self.send,
            try_process,
            _state: NeedsConnectionKind,
        }
    }
}
impl<In, Out, Data> OneshotMethodCallBuilder<In, Out, Data, NeedsConnectionKind>
where
    Data: Default + Copy,
{
    pub(crate) const fn kind(self, kind: DBusConnectionKind) -> OneshotMethodCall<In, Out, Data> {
        OneshotMethodCall::new_without_data(self.send, self.try_process, kind)
    }
}
