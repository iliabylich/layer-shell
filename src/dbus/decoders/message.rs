use crate::dbus::{
    decoders::{DecodingBuffer, HeaderDecoder, ValueDecoder, signature::SignatureDecoder},
    types::{CompleteType, Header, HeaderFieldName, Message, MessageType, Value},
};
use anyhow::{Context, Result, bail};
use std::borrow::Cow;

pub(crate) struct MessageDecoder;

impl MessageDecoder {
    pub(crate) fn decode(bytes: &[u8]) -> Result<Message<'static>> {
        let mut buf = DecodingBuffer::new(bytes);
        let header = HeaderDecoder::decode(&mut buf)?;

        let mut path = None;
        let mut interface = None;
        let mut member = None;
        let mut error_name = None;
        let mut reply_serial = None;
        let mut destination = None;
        let mut sender = None;
        let mut signature = None;
        let mut unix_fds = None;

        let len = buf.next_u32()?;
        let end = buf.pos() + len as usize;
        let header_field_type =
            CompleteType::Struct(vec![CompleteType::Byte, CompleteType::Variant]);

        while buf.pos() < end {
            buf.align(8)?;
            let header_field =
                ValueDecoder::decode_value_by_complete_type(&mut buf, &header_field_type)?;

            let Value::Struct(pair) = header_field else {
                bail!("got {header_field:?} instead of a header field struct");
            };

            let [header_field_name, value]: [_; 2] = pair.try_into().map_err(|not_a_pair| {
                anyhow::anyhow!("expected two elements, got {not_a_pair:?}")
            })?;

            let Value::Byte(header_field_name) = header_field_name else {
                bail!("got {header_field_name:?} instead of a header field name");
            };
            let header_field_name = HeaderFieldName::from(header_field_name);

            let Value::Variant(value) = value else {
                bail!("got {value:?} instead of Variant in a header field");
            };

            match (header_field_name, *value) {
                (HeaderFieldName::Path, Value::ObjectPath(value)) => {
                    path = Some(value);
                }
                (HeaderFieldName::Interface, Value::String(value)) => {
                    interface = Some(Cow::Owned(value));
                }
                (HeaderFieldName::Member, Value::String(value)) => {
                    member = Some(Cow::Owned(value));
                }
                (HeaderFieldName::ErrorName, Value::String(value)) => {
                    error_name = Some(Cow::Owned(value));
                }
                (HeaderFieldName::ReplySerial, Value::UInt32(value)) => {
                    reply_serial = Some(value);
                }
                (HeaderFieldName::Destination, Value::String(value)) => {
                    destination = Some(Cow::Owned(value));
                }
                (HeaderFieldName::Sender, Value::String(value)) => {
                    sender = Some(Cow::Owned(value));
                }
                (HeaderFieldName::Signature, Value::Signature(value)) => {
                    let mut buf = DecodingBuffer::new(&value);
                    signature = Some(SignatureDecoder::decode_signature(&mut buf)?);
                }
                (HeaderFieldName::UnixFds, Value::UInt32(value)) => {
                    unix_fds = Some(value);
                }
                (header_field_name, value) => {
                    bail!(
                        "invalid combination of header field name/value: {header_field_name:?} vs {value:?}"
                    );
                }
            }
        }

        let mut body = vec![];
        if let Some(signature) = signature.as_ref()
            && !signature.items.is_empty()
        {
            buf.align(8)?;
            body = ValueDecoder::decode_values_by_signature(&mut buf, signature)?;
        }

        build_message(
            header,
            path,
            interface,
            member,
            error_name,
            reply_serial,
            destination,
            sender,
            unix_fds,
            body,
        )
    }
}

#[expect(clippy::too_many_arguments)]
fn build_message<'a>(
    header: Header,
    path: Option<Cow<'a, str>>,
    interface: Option<Cow<'a, str>>,
    member: Option<Cow<'a, str>>,
    error_name: Option<Cow<'a, str>>,
    reply_serial: Option<u32>,
    destination: Option<Cow<'a, str>>,
    sender: Option<Cow<'a, str>>,
    unix_fds: Option<u32>,
    body: Vec<Value<'a>>,
) -> Result<Message<'a>> {
    match header.message_type {
        MessageType::MethodCall => {
            let path = path.context("MethodCall missing path")?;
            let member = member.context("MethodCall missing member")?;

            if error_name.is_some() {
                bail!("MethodCall should not have error_name");
            }
            if reply_serial.is_some() {
                bail!("MethodCall should not have reply_serial");
            }

            Ok(Message::MethodCall {
                serial: header.serial,
                path,
                member,
                interface,
                destination,
                sender,
                unix_fds,
                body,
            })
        }
        MessageType::MethodReturn => {
            let reply_serial = reply_serial.context("MethodReturn missing reply_serial")?;

            if path.is_some() {
                bail!("MethodReturn should not have path");
            }
            if member.is_some() {
                bail!("MethodReturn should not have member");
            }
            if interface.is_some() {
                bail!("MethodReturn should not have interface");
            }
            if error_name.is_some() {
                bail!("MethodReturn should not have error_name");
            }

            Ok(Message::MethodReturn {
                serial: header.serial,
                reply_serial,
                destination,
                sender,
                unix_fds,
                body,
            })
        }
        MessageType::Error => {
            let error_name = error_name.context("Error missing error_name")?;
            let reply_serial = reply_serial.context("Error missing reply_serial")?;

            if path.is_some() {
                bail!("Error should not have path");
            }
            if member.is_some() {
                bail!("Error should not have member");
            }
            if interface.is_some() {
                bail!("Error should not have interface");
            }

            Ok(Message::Error {
                serial: header.serial,
                error_name,
                reply_serial,
                destination,
                sender,
                unix_fds,
                body,
            })
        }
        MessageType::Signal => {
            let path = path.context("Signal missing path")?;
            let interface = interface.context("Signal missing interface")?;
            let member = member.context("Signal missing member")?;

            if error_name.is_some() {
                bail!("Signal should not have error_name");
            }
            if reply_serial.is_some() {
                bail!("Signal should not have reply_serial");
            }

            Ok(Message::Signal {
                serial: header.serial,
                path,
                interface,
                member,
                destination,
                sender,
                unix_fds,
                body,
            })
        }
        MessageType::Invalid => {
            bail!("Invalid message type")
        }
    }
}
