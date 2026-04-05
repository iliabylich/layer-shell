use crate::{
    dbus::types::{MessageType, Value},
    utils::StringRef,
};

#[derive(Debug, PartialEq)]
pub(crate) enum OutgoingMessage {
    MethodCall {
        destination: Option<StringRef>,
        path: StringRef,
        interface: Option<StringRef>,
        serial: u32,
        member: StringRef,
        sender: Option<StringRef>,
        unix_fds: Option<u32>,
        body: Vec<Value>,
    },
    MethodReturn {
        serial: u32,
        reply_serial: u32,
        destination: Option<StringRef>,
        sender: Option<StringRef>,
        unix_fds: Option<u32>,
        body: Vec<Value>,
    },
    Error {
        serial: u32,
        error_name: StringRef,
        reply_serial: u32,
        destination: Option<StringRef>,
        sender: Option<StringRef>,
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

    pub(crate) fn path(&self) -> Option<StringRef> {
        match self {
            Self::MethodCall { path, .. } => Some(path.clone()),
            _ => None,
        }
    }

    pub(crate) fn member(&self) -> Option<StringRef> {
        match self {
            Self::MethodCall { member, .. } => Some(member.clone()),
            _ => None,
        }
    }

    pub(crate) fn interface(&self) -> Option<StringRef> {
        match self {
            Self::MethodCall { interface, .. } => interface.clone(),
            _ => None,
        }
    }

    pub(crate) fn error_name(&self) -> Option<StringRef> {
        match self {
            Self::Error { error_name, .. } => Some(error_name.clone()),
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

    pub(crate) fn destination(&self) -> Option<StringRef> {
        match self {
            Self::MethodCall { destination, .. }
            | Self::MethodReturn { destination, .. }
            | Self::Error { destination, .. } => destination.clone(),
        }
    }

    pub(crate) fn sender(&self) -> Option<StringRef> {
        match self {
            Self::MethodCall { sender, .. }
            | Self::MethodReturn { sender, .. }
            | Self::Error { sender, .. } => sender.clone(),
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
            destination: Some(StringRef::new(destination)),
            sender: None,
            unix_fds: None,
            body: vec![],
        }
    }

    pub(crate) fn new_err_no_method(reply_serial: u32, destination: &str) -> Self {
        OutgoingMessage::Error {
            serial: 0,
            error_name: StringRef::new("org.freedesktop.DBus.Error.UnknownMethod"),
            reply_serial,
            destination: Some(StringRef::new(destination)),
            sender: None,
            unix_fds: None,
            body: vec![Value::StringRef(StringRef::new("Unknown method"))],
        }
    }
}
