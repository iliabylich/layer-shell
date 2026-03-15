use crate::dbus::decoder::Cursor;
use anyhow::{Result, bail, ensure};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HeaderFieldCode {
    Invalid = 0,
    Path = 1,
    Interface = 2,
    Member = 3,
    ErrorName = 4,
    ReplySerial = 5,
    Destination = 6,
    Sender = 7,
    Signature = 8,
    UnixFds = 9,
}
impl HeaderFieldCode {
    fn cut(cur: &mut Cursor<'_>) -> Result<Self> {
        let code = cur.cut_u8()?;
        let code = match code {
            0 => Self::Invalid,
            1 => Self::Path,
            2 => Self::Interface,
            3 => Self::Member,
            4 => Self::ErrorName,
            5 => Self::ReplySerial,
            6 => Self::Destination,
            7 => Self::Sender,
            8 => Self::Signature,
            9 => Self::UnixFds,
            other => bail!("unknown header field code: {other}"),
        };
        Ok(code)
    }
}

#[derive(Debug)]
enum HeaderField<'a> {
    #[allow(dead_code)]
    Invalid,
    Path(&'a str),
    Interface(&'a str),
    Member(&'a str),
    ErrorName(&'a str),
    ReplySerial(u32),
    Destination(&'a str),
    Sender(&'a str),
    Signature(&'a str),
    UnixFds(u32),
}
impl<'a> HeaderField<'a> {
    fn cut(cur: &mut Cursor<'a>) -> Result<Self> {
        cur.align(8)?;
        let code = HeaderFieldCode::cut(cur)?;
        let sig = cur.cut_signature()?;
        macro_rules! ensure_header_sig {
            ($expected:literal, $field:literal) => {
                ensure!(
                    sig == $expected,
                    "header field {} expects variant sig {:?}, got {:?}",
                    $field,
                    $expected,
                    sig
                );
            };
        }
        match code {
            HeaderFieldCode::Invalid => unreachable!(),

            HeaderFieldCode::Path => {
                ensure_header_sig!("o", "Path");
                let path = cur.cut_string()?;
                Ok(Self::Path(path))
            }
            HeaderFieldCode::Interface => {
                ensure_header_sig!("s", "Interface");
                let interface = cur.cut_string()?;
                Ok(Self::Interface(interface))
            }
            HeaderFieldCode::Member => {
                ensure_header_sig!("s", "Member");
                let member = cur.cut_string()?;
                Ok(Self::Member(member))
            }
            HeaderFieldCode::ErrorName => {
                ensure_header_sig!("s", "ErrorName");
                let error_name = cur.cut_string()?;
                Ok(Self::ErrorName(error_name))
            }
            HeaderFieldCode::ReplySerial => {
                ensure_header_sig!("u", "ReplySerial");
                let reply_serial = cur.cut_u32()?;
                Ok(Self::ReplySerial(reply_serial))
            }
            HeaderFieldCode::Destination => {
                ensure_header_sig!("s", "Destination");
                let destination = cur.cut_string()?;
                Ok(Self::Destination(destination))
            }
            HeaderFieldCode::Sender => {
                ensure_header_sig!("s", "Sender");
                let sender = cur.cut_string()?;
                Ok(Self::Sender(sender))
            }
            HeaderFieldCode::Signature => {
                ensure_header_sig!("g", "Signature");
                let signature = cur.cut_signature()?;
                Ok(Self::Signature(signature))
            }
            HeaderFieldCode::UnixFds => {
                ensure_header_sig!("u", "UnixFds");
                let unix_fds = cur.cut_u32()?;
                Ok(Self::UnixFds(unix_fds))
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct HeaderFields<'a> {
    pub(crate) path: Option<&'a str>,
    pub(crate) interface: Option<&'a str>,
    pub(crate) member: Option<&'a str>,
    pub(crate) error_name: Option<&'a str>,
    pub(crate) reply_serial: Option<u32>,
    pub(crate) destination: Option<&'a str>,
    pub(crate) sender: Option<&'a str>,
    pub(crate) signature: Option<&'a str>,
    pub(crate) unix_fds: Option<u32>,
}
impl<'a> HeaderFields<'a> {
    pub(crate) fn cut(mut headers: Cursor<'a>) -> Result<Self> {
        let mut this = Self {
            path: None,
            interface: None,
            member: None,
            error_name: None,
            reply_serial: None,
            destination: None,
            sender: None,
            signature: None,
            unix_fds: None,
        };

        while !headers.buf().is_empty() {
            let header = HeaderField::cut(&mut headers)?;

            match header {
                HeaderField::Invalid => bail!("received Invalid header field"),
                HeaderField::Path(path) => this.path = Some(path),
                HeaderField::Interface(interface) => this.interface = Some(interface),
                HeaderField::Member(member) => this.member = Some(member),
                HeaderField::ErrorName(error_name) => this.error_name = Some(error_name),
                HeaderField::ReplySerial(reply_serial) => this.reply_serial = Some(reply_serial),
                HeaderField::Destination(destination) => this.destination = Some(destination),
                HeaderField::Sender(sender) => this.sender = Some(sender),
                HeaderField::Signature(signature) => this.signature = Some(signature),
                HeaderField::UnixFds(unix_fds) => this.unix_fds = Some(unix_fds),
            }
        }

        Ok(this)
    }
}
