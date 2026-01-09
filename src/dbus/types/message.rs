use crate::dbus::types::{MessageType, Value};
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub(crate) enum Message {
    MethodCall {
        serial: u32,
        path: Cow<'static, str>,
        member: Cow<'static, str>,
        interface: Option<Cow<'static, str>>,
        destination: Option<Cow<'static, str>>,
        sender: Option<Cow<'static, str>>,
        unix_fds: Option<u32>,
        body: Vec<Value>,
    },
    MethodReturn {
        serial: u32,
        reply_serial: u32,
        destination: Option<Cow<'static, str>>,
        sender: Option<Cow<'static, str>>,
        unix_fds: Option<u32>,
        body: Vec<Value>,
    },
    Error {
        serial: u32,
        error_name: String,
        reply_serial: u32,
        destination: Option<Cow<'static, str>>,
        sender: Option<Cow<'static, str>>,
        unix_fds: Option<u32>,
        body: Vec<Value>,
    },
    Signal {
        serial: u32,
        path: Cow<'static, str>,
        interface: Cow<'static, str>,
        member: Cow<'static, str>,
        destination: Option<Cow<'static, str>>,
        sender: Option<Cow<'static, str>>,
        unix_fds: Option<u32>,
        body: Vec<Value>,
    },
}

impl Message {
    pub(crate) fn serial(&self) -> u32 {
        match self {
            Self::MethodCall { serial, .. }
            | Self::MethodReturn { serial, .. }
            | Self::Error { serial, .. }
            | Self::Signal { serial, .. } => *serial,
        }
    }

    pub(crate) fn serial_mut(&mut self) -> &mut u32 {
        match self {
            Self::MethodCall { serial, .. }
            | Self::MethodReturn { serial, .. }
            | Self::Error { serial, .. }
            | Self::Signal { serial, .. } => serial,
        }
    }

    pub(crate) fn message_type(&self) -> MessageType {
        match self {
            Self::MethodCall { .. } => MessageType::MethodCall,
            Self::MethodReturn { .. } => MessageType::MethodReturn,
            Self::Error { .. } => MessageType::Error,
            Self::Signal { .. } => MessageType::Signal,
        }
    }

    pub(crate) fn path(&self) -> Option<Cow<'static, str>> {
        match self {
            Self::MethodCall { path, .. } | Self::Signal { path, .. } => Some(path.clone()),
            _ => None,
        }
    }

    pub(crate) fn member(&self) -> Option<&str> {
        match self {
            Self::MethodCall { member, .. } | Self::Signal { member, .. } => Some(member),
            _ => None,
        }
    }

    pub(crate) fn interface(&self) -> Option<&str> {
        match self {
            Self::MethodCall { interface, .. } => interface.as_deref(),
            Self::Signal { interface, .. } => Some(interface),
            _ => None,
        }
    }

    pub(crate) fn error_name(&self) -> Option<&str> {
        match self {
            Self::Error { error_name, .. } => Some(error_name),
            _ => None,
        }
    }

    pub(crate) fn reply_serial(&self) -> Option<u32> {
        match self {
            Self::MethodReturn { reply_serial, .. } | Self::Error { reply_serial, .. } => {
                Some(*reply_serial)
            }
            _ => None,
        }
    }

    pub(crate) fn destination(&self) -> Option<Cow<'static, str>> {
        match self {
            Self::MethodCall { destination, .. }
            | Self::MethodReturn { destination, .. }
            | Self::Error { destination, .. }
            | Self::Signal { destination, .. } => destination.clone(),
        }
    }

    pub(crate) fn sender(&self) -> Option<&str> {
        match self {
            Self::MethodCall { sender, .. }
            | Self::MethodReturn { sender, .. }
            | Self::Error { sender, .. }
            | Self::Signal { sender, .. } => sender.as_deref(),
        }
    }

    pub(crate) fn body(&self) -> &[Value] {
        match self {
            Self::MethodCall { body, .. }
            | Self::MethodReturn { body, .. }
            | Self::Error { body, .. }
            | Self::Signal { body, .. } => body,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn body_mut(&mut self) -> &mut Vec<Value> {
        match self {
            Self::MethodCall { body, .. }
            | Self::MethodReturn { body, .. }
            | Self::Error { body, .. }
            | Self::Signal { body, .. } => body,
        }
    }

    pub(crate) fn unix_fds(&self) -> Option<u32> {
        match self {
            Self::MethodCall { unix_fds, .. }
            | Self::MethodReturn { unix_fds, .. }
            | Self::Error { unix_fds, .. }
            | Self::Signal { unix_fds, .. } => *unix_fds,
        }
    }
}
