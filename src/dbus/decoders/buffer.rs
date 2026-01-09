use anyhow::{Context, Result};

pub(crate) struct DecodingBuffer<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> DecodingBuffer<'a> {
    pub(crate) fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    pub(crate) fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    pub(crate) fn pos(&self) -> usize {
        self.pos
    }

    pub(crate) fn peek(&self) -> Option<u8> {
        self.buf.get(self.pos).copied()
    }

    pub(crate) fn peek_u32(&self) -> Option<u32> {
        Some(u32::from_le_bytes([
            self.buf.get(self.pos).copied()?,
            self.buf.get(self.pos + 1).copied()?,
            self.buf.get(self.pos + 2).copied()?,
            self.buf.get(self.pos + 3).copied()?,
        ]))
    }

    pub(crate) fn next_u8(&mut self) -> Result<u8> {
        let byte = self.buf.get(self.pos).context("EOF")?;
        self.pos += 1;
        Ok(*byte)
    }

    pub(crate) fn next_u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes([self.next_u8()?, self.next_u8()?]))
    }

    pub(crate) fn next_u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes([
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
        ]))
    }

    pub(crate) fn next_u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes([
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
        ]))
    }

    pub(crate) fn next_i16(&mut self) -> Result<i16> {
        Ok(i16::from_le_bytes([self.next_u8()?, self.next_u8()?]))
    }

    pub(crate) fn next_i32(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes([
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
        ]))
    }

    pub(crate) fn next_i64(&mut self) -> Result<i64> {
        Ok(i64::from_le_bytes([
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
        ]))
    }

    pub(crate) fn next_f64(&mut self) -> Result<f64> {
        Ok(f64::from_le_bytes([
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
        ]))
    }

    pub(crate) fn next_n(&mut self, count: usize) -> Result<&[u8]> {
        let bytes = self.buf.get(self.pos..self.pos + count).context("EOF")?;
        self.pos += count;
        Ok(bytes)
    }

    pub(crate) fn is_eof(&self) -> bool {
        self.pos >= self.buf.len()
    }

    pub(crate) fn skip_n(&mut self, count: usize) {
        self.pos += count;
    }

    pub(crate) fn skip(&mut self) {
        self.skip_n(1)
    }

    pub(crate) fn align(&mut self, align: usize) -> Result<()> {
        self.set_pos(self.pos.next_multiple_of(align));
        Ok(())
    }
}

impl std::fmt::Debug for DecodingBuffer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DecodingBuffer")
            .field("rem", &&self.buf[self.pos..])
            .field("pos", &self.pos)
            .finish()
    }
}
