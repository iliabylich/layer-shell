pub struct ArrayWriter<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> ArrayWriter<'a> {
    pub(crate) const fn new(buf: &'a mut [u8]) -> Self {
        ArrayWriter { buf, offset: 0 }
    }

    pub(crate) const fn as_bytes(&self) -> &[u8] {
        let (head, _tail) = self.buf.split_at(self.offset);
        head
    }

    pub(crate) const fn offset(&self) -> usize {
        self.offset
    }
}

impl core::fmt::Write for ArrayWriter<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();

        let remainder = self.buf.get_mut(self.offset..).ok_or(core::fmt::Error)?;
        if remainder.len() < bytes.len() {
            return Err(core::fmt::Error);
        }
        let remainder = remainder.get_mut(..bytes.len()).ok_or(core::fmt::Error)?;
        remainder.copy_from_slice(bytes);

        self.offset += bytes.len();
        Ok(())
    }
}

macro_rules! write_in_place {
    ($buf:expr, $($arg:tt)*) => {{
        use core::fmt::Write;
        let mut writer = $crate::utils::ArrayWriter::new($buf);
        write!(&mut writer, $($arg)+).unwrap_or_else(|_| {
            panic!("formatted data doesn't fit into fixed size array");
        });
        let offset = writer.offset();
        let (head, _tail) = $buf.split_at_checked(offset).unwrap_or_else(|| {
            panic!("impl Write for ArrayWriter has a bug, computed offset is too large");
        });
        core::str::from_utf8(head).unwrap_or_else(|err| {
            panic!("non-utf8 in-place formatter string: {err:?}")
        })
    }};
}
pub(crate) use write_in_place;
