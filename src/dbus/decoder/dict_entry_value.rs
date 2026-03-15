use crate::dbus::decoder::{CompleteType, Cursor, Value};
use anyhow::{Result, ensure};

#[derive(Debug)]
pub(crate) struct DictEntryValue<'a> {
    key_type: CompleteType<'a>,
    value_type: CompleteType<'a>,
    cur: Cursor<'a>,
}

impl<'a> DictEntryValue<'a> {
    pub(crate) fn cut(cur: &mut Cursor<'a>, key_sig: &'a str, value_sig: &'a str) -> Result<Self> {
        let (key_type, leftover) = CompleteType::cut(key_sig)?;
        ensure!(leftover.is_empty(), "expected no leftover");

        let (value_type, leftover) = CompleteType::cut(value_sig)?;
        ensure!(leftover.is_empty(), "expected no leftover");

        Ok(Self {
            key_type,
            value_type,
            cur: *cur,
        })
    }

    pub(crate) fn key_value(&self) -> Result<(Value<'a>, Value<'a>)> {
        let mut cur = self.cur;
        let key = Value::cut(&mut cur, self.key_type)?;
        let value = Value::cut(&mut cur, self.value_type)?;
        ensure!(cur.buf().is_empty(), "expected not leftover");
        Ok((key, value))
    }
}
