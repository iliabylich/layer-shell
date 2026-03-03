pub(crate) struct DnsName {
    buf: [u8; 253],
    len: usize,
}

impl DnsName {
    pub(crate) fn new() -> Self {
        DnsName {
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

impl std::fmt::Debug for DnsName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DNS({:?})", std::str::from_utf8(self.as_bytes()))
    }
}
