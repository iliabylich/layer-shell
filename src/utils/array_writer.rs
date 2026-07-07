use anyhow::{Context, Result};

pub(crate) struct ArrayWriter<'a> {
    pub(crate) buf: &'a mut [u8],
    pub(crate) offset: usize,
}

impl<'a> ArrayWriter<'a> {
    pub(crate) const fn new(buf: &'a mut [u8]) -> Self {
        ArrayWriter { buf, offset: 0 }
    }

    pub(crate) fn as_str(&self) -> Result<&str> {
        core::str::from_utf8(self.buf.get(..self.offset).context("malformed offset")?)
            .context("malformed ArrayWriter's buffer")
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
