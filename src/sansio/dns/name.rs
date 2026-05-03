pub(crate) struct DnsName {
    buf: [u8; 253],
    len: usize,
}

#[expect(clippy::indexing_slicing)]
impl DnsName {
    pub(crate) const fn new() -> Self {
        Self {
            buf: [0u8; 253],
            len: 0,
        }
    }

    pub(crate) fn push_label(&mut self, label: &[u8]) {
        if self.len > 0 {
            self.buf[self.len] = b'.';
            self.len += 1;
        }
        self.buf[self.len..self.len + label.len()].copy_from_slice(label);
        self.len += label.len();
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl core::fmt::Debug for DnsName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DNS({:?})", core::str::from_utf8(self.as_bytes()))
    }
}
