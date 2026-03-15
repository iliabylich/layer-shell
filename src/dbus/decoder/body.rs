use crate::dbus::decoder::{CompleteType, Cursor, Value};
use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Body<'a> {
    signature: &'a str,
    cur: Cursor<'a>,
}

impl<'a> Body<'a> {
    pub(crate) fn new(signature: &'a str, cur: Cursor<'a>) -> Self {
        Self { signature, cur }
    }

    pub(crate) fn try_next(&mut self) -> Result<Option<Value<'a>>> {
        if self.signature.is_empty() && self.cur.buf().is_empty() {
            return Ok(None);
        }

        let (type_, remainder) = CompleteType::cut(self.signature)?;
        self.signature = remainder;

        let value = Value::cut(&mut self.cur, type_)?;
        Ok(Some(value))
    }
}
