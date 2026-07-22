pub struct FixedSizeBuffer<const N: usize> {
    buf: [u8; N],
    pos: usize,
}

impl<const N: usize> FixedSizeBuffer<N> {
    pub(crate) const fn new() -> Self {
        Self {
            buf: [0; _],
            pos: 0,
        }
    }

    pub(crate) fn remainder(&mut self) -> &mut [u8] {
        &mut self.buf[self.pos..]
    }

    pub(crate) const fn written(&mut self, count: usize) -> Option<[u8; N]> {
        self.pos += count;
        if self.pos == N {
            let out = self.buf;
            self.buf = [0; _];
            self.pos = 0;
            Some(out)
        } else {
            None
        }
    }
}
