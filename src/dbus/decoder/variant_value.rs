use crate::dbus::decoder::{CompleteType, Cursor, Value};
use anyhow::{Result, ensure};

#[derive(Debug)]
pub(crate) struct VariantValue<'a> {
    cur: Cursor<'a>,
}

impl<'a> VariantValue<'a> {
    pub(crate) fn new(cur: Cursor<'a>) -> Self {
        Self { cur }
    }

    pub(crate) fn materialize(&self) -> Result<Value<'a>> {
        let mut cur = self.cur;
        let signature = cur.cut_signature()?;

        let (type_, leftover) = CompleteType::cut(signature)?;
        ensure!(leftover.is_empty(), "expected no leftover");

        let value = Value::cut(&mut cur, type_)?;
        ensure!(cur.buf().is_empty(), "expected no leftover");

        Ok(value)
    }
}
