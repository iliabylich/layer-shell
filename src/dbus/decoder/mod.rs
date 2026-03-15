mod array_value;
mod body;
mod complete_type;
mod cursor;
mod dict_entry_value;
mod header;
mod header_fields;
mod message_type;
mod struct_value;
mod value;
mod variant_value;

use anyhow::{Context as _, Result};
pub(crate) use array_value::ArrayValue;
pub(crate) use body::Body;
pub(crate) use complete_type::{CompleteType, CompleteTypeStructFieldsIter};
pub(crate) use cursor::Cursor;
pub(crate) use dict_entry_value::DictEntryValue;
pub(crate) use header::Header;
pub(crate) use header_fields::HeaderFields;
pub(crate) use message_type::MessageType;
pub(crate) use struct_value::StructValue;
pub(crate) use value::Value;
pub(crate) use variant_value::VariantValue;

#[derive(Clone, Copy)]
pub(crate) struct IncomingMessage<'a> {
    pub(crate) message_type: MessageType,
    pub(crate) serial: u32,

    pub(crate) path: Option<&'a str>,
    pub(crate) interface: Option<&'a str>,
    pub(crate) member: Option<&'a str>,
    pub(crate) error_name: Option<&'a str>,
    pub(crate) reply_serial: Option<u32>,
    pub(crate) destination: Option<&'a str>,
    pub(crate) sender: Option<&'a str>,
    pub(crate) signature: Option<&'a str>,
    pub(crate) unix_fds: Option<u32>,

    pub(crate) body: Option<Body<'a>>,
}

impl<'a> IncomingMessage<'a> {
    pub(crate) fn new(buf: &'a [u8]) -> Result<Self> {
        let mut cur = Cursor::new(buf, 0);

        let Header {
            _endian,
            message_type,
            _flags,
            _protocol_version,
            body_len,
            serial,
            header_fields_len,
        } = Header::cut(&mut cur)?;
        let message_type = MessageType::from_u8(message_type)?;

        let headers = cur.take(header_fields_len as usize)?;

        let HeaderFields {
            path,
            interface,
            member,
            error_name,
            reply_serial,
            destination,
            sender,
            signature,
            unix_fds,
        } = HeaderFields::cut(Cursor::new(headers, 0))?;

        let mut body = None;
        if let Some(signature) = signature {
            let body_padding = (8 - (header_fields_len as usize % 8)) % 8;
            let body_buf = cur
                .buf()
                .get(body_padding..body_padding + body_len as usize)
                .context("malformed body: truncated bytes")?;
            body = Some(Body::new(signature, Cursor::new(body_buf, 0)));
        }

        Ok(Self {
            message_type,
            serial,

            path,
            interface,
            member,
            error_name,
            reply_serial,
            destination,
            sender,
            signature,
            unix_fds,

            body,
        })
    }

    #[allow(dead_code)]
    pub(crate) fn log(&self) -> Result<()> {
        eprintln!("============");
        eprintln!("Type = {:?}", self.message_type);
        eprintln!("Serial = {}", self.serial);
        eprintln!("Path = {:?}", self.path);
        eprintln!("Interface = {:?}", self.interface);
        eprintln!("Member = {:?}", self.member);
        eprintln!("ErrorName = {:?}", self.error_name);
        eprintln!("ReplySerial = {:?}", self.reply_serial);
        eprintln!("Destination = {:?}", self.destination);
        eprintln!("Sender = {:?}", self.sender);
        eprintln!("Signature = {:?}", self.signature);
        eprintln!("UnixFDs = {:?}", self.unix_fds);

        if let Some(mut body) = self.body {
            eprintln!("Body:");
            while let Some(value) = body.try_next()? {
                value.log(4)?;
            }
        }
        eprintln!("============");

        Ok(())
    }
}
