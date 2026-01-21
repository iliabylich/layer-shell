use crate::dbus::{
    encoders::{EncodingBuffer, HeaderEncoder, SignatureEncoder, ValueEncoder},
    types::{Flags, HeaderFieldName, Message, Signature, Value},
};
use std::borrow::Cow;

pub(crate) struct MessageEncoder;

impl MessageEncoder {
    pub(crate) fn encode(message: &Message) -> Vec<u8> {
        let mut buf = EncodingBuffer::new();

        HeaderEncoder::encode(
            &mut buf,
            message.message_type() as u8,
            Flags::default().into(),
            message.serial(),
        );

        buf.encode_u32(0); // header fields len
        let header_fields_start = buf.size();
        {
            if let Some(path) = message.path() {
                buf.align(8);
                ValueEncoder::encode_header(
                    &mut buf,
                    HeaderFieldName::Path,
                    &Value::ObjectPath(Cow::Owned(path.to_string())),
                );
            }
            if let Some(interface) = message.interface() {
                buf.align(8);
                ValueEncoder::encode_header(
                    &mut buf,
                    HeaderFieldName::Interface,
                    &Value::String(interface.to_string()),
                );
            }
            if let Some(member) = message.member() {
                buf.align(8);
                ValueEncoder::encode_header(
                    &mut buf,
                    HeaderFieldName::Member,
                    &Value::String(member.to_string()),
                );
            }
            if let Some(error_name) = message.error_name() {
                buf.align(8);
                ValueEncoder::encode_header(
                    &mut buf,
                    HeaderFieldName::ErrorName,
                    &Value::String(error_name.to_string()),
                );
            }
            if let Some(reply_serial) = message.reply_serial() {
                buf.align(8);
                ValueEncoder::encode_header(
                    &mut buf,
                    HeaderFieldName::ReplySerial,
                    &Value::UInt32(reply_serial),
                );
            }
            if let Some(destination) = message.destination() {
                buf.align(8);
                ValueEncoder::encode_header(
                    &mut buf,
                    HeaderFieldName::Destination,
                    &Value::String(destination.to_string()),
                );
            }
            if let Some(sender) = message.sender() {
                buf.align(8);
                ValueEncoder::encode_header(
                    &mut buf,
                    HeaderFieldName::Sender,
                    &Value::String(sender.to_string()),
                );
            }
            if let Some(unix_fds) = message.unix_fds() {
                buf.align(8);
                ValueEncoder::encode_header(
                    &mut buf,
                    HeaderFieldName::UnixFds,
                    &Value::UInt32(unix_fds),
                );
            }

            let body = message.body();
            if !body.is_empty() {
                buf.align(8);
                let signature = Signature {
                    items: body.iter().map(|v| v.complete_type()).collect(),
                };
                let mut sig_buf = EncodingBuffer::new();
                SignatureEncoder::encode_signature(&mut sig_buf, &signature);
                let sig_buf = sig_buf.done();
                ValueEncoder::encode_header(
                    &mut buf,
                    HeaderFieldName::Signature,
                    &Value::Signature(sig_buf),
                );
            }
        };
        let header_fieldss_end = buf.size();

        buf.set_u32(12, (header_fieldss_end - header_fields_start) as u32);
        buf.align(8);

        let body_starts_at = buf.size();
        for value in message.body() {
            ValueEncoder::encode_value(&mut buf, value);
        }
        let body_len = buf.size() - body_starts_at;
        buf.set_u32(4, body_len as u32);

        buf.done()
    }
}
