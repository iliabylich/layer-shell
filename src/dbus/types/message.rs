use crate::{
    dbus::types::{MessageType, Value},
    ffi::ShortString,
};

#[derive(Debug, PartialEq)]
#[expect(clippy::large_enum_variant)]
pub(crate) enum OutgoingMessage {
    MethodCall {
        destination: Option<ShortString>,
        path: ShortString,
        interface: Option<ShortString>,
        serial: u32,
        member: ShortString,
        sender: Option<ShortString>,
        unix_fds: Option<u32>,
        body: Vec<Value>,
    },
    MethodReturn {
        serial: u32,
        reply_serial: u32,
        destination: Option<ShortString>,
        sender: Option<ShortString>,
        unix_fds: Option<u32>,
        body: Vec<Value>,
    },
    Error {
        serial: u32,
        error_name: ShortString,
        reply_serial: u32,
        destination: Option<ShortString>,
        sender: Option<ShortString>,
        unix_fds: Option<u32>,
        body: Vec<Value>,
    },
}

impl OutgoingMessage {
    pub(crate) fn serial(&self) -> u32 {
        match self {
            Self::MethodCall { serial, .. }
            | Self::MethodReturn { serial, .. }
            | Self::Error { serial, .. } => *serial,
        }
    }

    pub(crate) fn serial_mut(&mut self) -> &mut u32 {
        match self {
            Self::MethodCall { serial, .. }
            | Self::MethodReturn { serial, .. }
            | Self::Error { serial, .. } => serial,
        }
    }

    pub(crate) fn message_type(&self) -> MessageType {
        match self {
            Self::MethodCall { .. } => MessageType::MethodCall,
            Self::MethodReturn { .. } => MessageType::MethodReturn,
            Self::Error { .. } => MessageType::Error,
        }
    }

    pub(crate) fn path(&self) -> Option<ShortString> {
        match self {
            Self::MethodCall { path, .. } => Some(*path),
            _ => None,
        }
    }

    pub(crate) fn member(&self) -> Option<ShortString> {
        match self {
            Self::MethodCall { member, .. } => Some(*member),
            _ => None,
        }
    }

    pub(crate) fn interface(&self) -> Option<ShortString> {
        match self {
            Self::MethodCall {
                interface: Some(interface),
                ..
            } => Some(*interface),
            _ => None,
        }
    }

    pub(crate) fn error_name(&self) -> Option<ShortString> {
        match self {
            Self::Error { error_name, .. } => Some(*error_name),
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

    pub(crate) fn destination(&self) -> Option<ShortString> {
        match self {
            Self::MethodCall { destination, .. }
            | Self::MethodReturn { destination, .. }
            | Self::Error { destination, .. } => *destination,
        }
    }

    pub(crate) fn sender(&self) -> Option<ShortString> {
        match self {
            Self::MethodCall { sender, .. }
            | Self::MethodReturn { sender, .. }
            | Self::Error { sender, .. } => Some(*sender.as_ref()?),
        }
    }

    pub(crate) fn body(&self) -> &[Value] {
        match self {
            Self::MethodCall { body, .. }
            | Self::MethodReturn { body, .. }
            | Self::Error { body, .. } => body,
        }
    }

    pub(crate) fn unix_fds(&self) -> Option<u32> {
        match self {
            Self::MethodCall { unix_fds, .. }
            | Self::MethodReturn { unix_fds, .. }
            | Self::Error { unix_fds, .. } => *unix_fds,
        }
    }

    pub(crate) fn new_method_return_no_body(reply_serial: u32, destination: &str) -> Self {
        OutgoingMessage::MethodReturn {
            serial: 0,
            reply_serial,
            destination: Some(ShortString::from(destination)),
            sender: None,
            unix_fds: None,
            body: vec![],
        }
    }

    pub(crate) fn new_err_no_method(reply_serial: u32, destination: &str) -> Self {
        OutgoingMessage::Error {
            serial: 0,
            error_name: ShortString::from("org.freedesktop.DBus.Error.UnknownMethod"),
            reply_serial,
            destination: Some(ShortString::from(destination)),
            sender: None,
            unix_fds: None,
            body: vec![Value::ShortString(ShortString::from("Unknown method"))],
        }
    }
}
