use anyhow::{Context, Result};

pub(crate) struct ArrayWriter<'a> {
    pub(crate) buf: &'a mut [u8],
    pub(crate) offset: usize,
}

impl<'a> ArrayWriter<'a> {
    pub(crate) fn new(buf: &'a mut [u8]) -> Self {
        ArrayWriter { buf, offset: 0 }
    }

    pub(crate) fn as_str(&self) -> Result<&str> {
        core::str::from_utf8(&self.buf[..self.offset]).context("malformed ArrayWriter's buffer")
    }
}

impl<'a> core::fmt::Write for ArrayWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();

        let remainder = &mut self.buf[self.offset..];
        if remainder.len() < bytes.len() {
            return Err(core::fmt::Error);
        }
        let remainder = &mut remainder[..bytes.len()];
        remainder.copy_from_slice(bytes);

        self.offset += bytes.len();
        Ok(())
    }
}
