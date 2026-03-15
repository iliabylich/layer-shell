use crate::dbus::decoder::{CompleteType, Cursor, Value};
use anyhow::{Result, ensure};

#[derive(Debug)]
pub(crate) struct ArrayValue<'a> {
    item_type: CompleteType<'a>,
    cur: Cursor<'a>,
}

impl<'a> ArrayValue<'a> {
    pub(crate) fn cut(cur: &mut Cursor<'a>, item_sig: &'a str) -> Result<Self> {
        let (item_type, leftover) = CompleteType::cut(item_sig)?;
        ensure!(leftover.is_empty(), "expected no leftover");

        let array_bytesize = cur.cut_u32()?;
        cur.align(item_type.alignment())?;
        let items_offset = cur.offset();
        let items_buf = cur.take(array_bytesize as usize)?;
        ensure!(cur.buf().is_empty(), "expected no leftover");

        Ok(Self {
            item_type,
            cur: Cursor::new(items_buf, items_offset),
        })
    }

    pub(crate) fn iter(&self) -> ArrayValueIter<'a> {
        ArrayValueIter {
            item_type: self.item_type,
            cur: self.cur,
        }
    }
}

pub(crate) struct ArrayValueIter<'a> {
    item_type: CompleteType<'a>,
    cur: Cursor<'a>,
}

impl<'a> ArrayValueIter<'a> {
    pub(crate) fn try_next(&mut self) -> Result<Option<Value<'a>>> {
        if self.cur.buf().is_empty() {
            return Ok(None);
        }

        let value = Value::cut(&mut self.cur, self.item_type)?;
        Ok(Some(value))
    }
}
