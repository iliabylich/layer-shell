pub(crate) struct ArrayWriter<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> ArrayWriter<'a> {
    pub(crate) fn new(buf: &'a mut [u8]) -> Self {
        ArrayWriter { buf, offset: 0 }
    }

    pub(crate) fn offset(&self) -> usize {
        self.offset
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
