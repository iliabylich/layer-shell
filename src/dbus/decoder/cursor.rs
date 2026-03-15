use anyhow::{Context as _, Result};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Cursor<'a> {
    buf: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(buf: &'a [u8], offset: usize) -> Self {
        Self { buf, offset }
    }

    pub(crate) fn buf(&self) -> &'a [u8] {
        self.buf
    }

    pub(crate) fn offset(&self) -> usize {
        self.offset
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub(crate) fn align(&mut self, alignment: usize) -> Result<()> {
        let pad = (alignment - (self.offset % alignment)) % alignment;
        self.buf = self
            .buf
            .get(pad..)
            .with_context(|| format!("malformed alignment: need pad {pad}"))?;
        self.offset += pad;
        Ok(())
    }

    pub(crate) fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let (head, tail) = self.buf.split_at_checked(n).context("malformed buffer")?;
        self.buf = tail;
        self.offset += n;
        Ok(head)
    }

    pub(crate) fn cut_bytes<const N: usize>(&mut self, alignment: usize) -> Result<[u8; N]> {
        self.align(alignment)?;
        self.take(N)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("malformed fixed-size value"))
    }

    pub(crate) fn cut_u8(&mut self) -> Result<u8> {
        Ok(self.cut_bytes::<1>(1)?[0])
    }

    pub(crate) fn cut_bool(&mut self) -> Result<bool> {
        Ok(self.cut_u32()? != 0)
    }

    pub(crate) fn cut_i16(&mut self) -> Result<i16> {
        Ok(i16::from_le_bytes(self.cut_bytes::<2>(2)?))
    }

    pub(crate) fn cut_u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.cut_bytes::<2>(2)?))
    }

    pub(crate) fn cut_i32(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.cut_bytes::<4>(4)?))
    }

    pub(crate) fn cut_u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.cut_bytes::<4>(4)?))
    }

    pub(crate) fn cut_i64(&mut self) -> Result<i64> {
        Ok(i64::from_le_bytes(self.cut_bytes::<8>(8)?))
    }

    pub(crate) fn cut_u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.cut_bytes::<8>(8)?))
    }

    pub(crate) fn cut_f64(&mut self) -> Result<f64> {
        Ok(f64::from_le_bytes(self.cut_bytes::<8>(8)?))
    }

    pub(crate) fn cut_signature(&mut self) -> Result<&'a str> {
        let len = self.cut_u8()? as usize;
        let sig = self.take(len)?;
        let sig = std::str::from_utf8(sig).context("non-utf8 signature")?;
        self.take(1)?;
        Ok(sig)
    }

    pub(crate) fn cut_string(&mut self) -> Result<&'a str> {
        let len = self.cut_u32()? as usize;
        let s = self.take(len).context("malformed string")?;
        let s = std::str::from_utf8(s).context("non-utf8 string")?;
        self.take(1)?;
        Ok(s)
    }
}
