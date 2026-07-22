pub struct NlSeparatedBuffer {
    buf: [u8; 1_024 * 5],
    pos: usize,
}

#[expect(clippy::indexing_slicing)]
impl NlSeparatedBuffer {
    pub(crate) const fn new() -> Self {
        Self {
            buf: [0; _],
            pos: 0,
        }
    }

    pub(crate) fn remainder(&mut self) -> &mut [u8] {
        &mut self.buf[self.pos..]
    }

    pub(crate) const fn written(&mut self, count: usize) {
        self.pos += count;
    }

    pub(crate) fn pre_nl(&self) -> Option<&[u8]> {
        let nl_idx = self.buf[..self.pos].iter().position(|b| *b == b'\n')?;
        let (head, _) = self.buf.split_at_checked(nl_idx)?;
        Some(head)
    }

    pub(crate) fn drop_pre_nl(&mut self) {
        let Some(nl_idx) = self.buf[..self.pos].iter().position(|b| *b == b'\n') else {
            return;
        };
        self.buf.copy_within((nl_idx + 1).., 0);
        self.pos -= nl_idx + 1;
    }
}
