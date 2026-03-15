use crate::dbus::decoder::{CompleteType, CompleteTypeStructFieldsIter, Cursor, Value};
use anyhow::Result;

#[derive(Debug)]
pub(crate) struct StructValue<'a> {
    struct_type: CompleteType<'a>,
    cur: Cursor<'a>,
}
impl<'a> StructValue<'a> {
    pub(crate) fn new(struct_type: CompleteType<'a>, cur: Cursor<'a>) -> Self {
        Self { struct_type, cur }
    }

    pub(crate) fn iter(&self) -> Result<StructValueIter<'a>> {
        Ok(StructValueIter {
            field_type_iter: CompleteTypeStructFieldsIter::new(self.struct_type.buf())?,
            cur: self.cur,
        })
    }
}
pub(crate) struct StructValueIter<'a> {
    field_type_iter: CompleteTypeStructFieldsIter<'a>,
    cur: Cursor<'a>,
}
impl<'a> StructValueIter<'a> {
    pub(crate) fn try_next(&mut self) -> Result<Option<Value<'a>>> {
        let Some(field_type) = self.field_type_iter.try_next()? else {
            return Ok(None);
        };

        let value = Value::cut(&mut self.cur, field_type)?;

        Ok(Some(value))
    }
}
