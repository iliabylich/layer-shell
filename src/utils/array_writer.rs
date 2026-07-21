pub(crate) struct ArrayWriter<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> ArrayWriter<'a> {
    pub(crate) const fn new(buf: &'a mut [u8]) -> Self {
        ArrayWriter { buf, offset: 0 }
    }

    pub(crate) const fn as_bytes(&self) -> &[u8] {
        // SAFETY: self.offset is changed in `impl Write` so it's guaranteed to be valid
        let (head, _tail) = unsafe { self.buf.split_at_unchecked(self.offset) };
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
        write!(&mut writer, $($arg)+).unwrap_or_else(|_| unreachable!());
        let offset = writer.offset();
        let (head, _tail) = $buf.split_at_checked(offset).unwrap_or_else(|| unreachable!("Write for ArrayWriter has a bug"));
        head
    }};
}
pub(crate) use write_in_place;
