use anyhow::Context as _;

#[derive(Debug)]
pub(crate) struct EncodingBuffer {
    buf: Vec<u8>,
}

impl EncodingBuffer {
    pub(crate) fn new() -> Self {
        Self { buf: vec![] }
    }

    pub(crate) fn size(&self) -> usize {
        self.buf.len()
    }

    pub(crate) fn align(&mut self, align: usize) {
        while !self.size().is_multiple_of(align) {
            self.encode_u8(0);
        }
    }

    pub(crate) fn encode_u8(&mut self, value: u8) {
        self.buf.push(value)
    }

    pub(crate) fn encode_u16(&mut self, value: u16) {
        self.encode_bytes(&value.to_le_bytes());
    }

    pub(crate) fn encode_i16(&mut self, value: i16) {
        self.encode_bytes(&value.to_le_bytes());
    }

    pub(crate) fn encode_u32(&mut self, value: u32) {
        self.encode_bytes(&value.to_le_bytes());
    }

    pub(crate) fn encode_i32(&mut self, value: i32) {
        self.encode_bytes(&value.to_le_bytes());
    }

    pub(crate) fn encode_u64(&mut self, value: u64) {
        self.encode_bytes(&value.to_le_bytes());
    }

    pub(crate) fn encode_i64(&mut self, value: i64) {
        self.encode_bytes(&value.to_le_bytes());
    }

    pub(crate) fn encode_bytes(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    pub(crate) fn encode_f64(&mut self, value: f64) {
        self.encode_bytes(&value.to_le_bytes());
    }

    pub(crate) fn done(self) -> Vec<u8> {
        self.buf
    }

    pub(crate) fn set_u8(&mut self, at: usize, value: u8) {
        *self
            .buf
            .get_mut(at)
            .context("out of bounds")
            .unwrap_or_else(|err| {
                eprintln!("{err:?}");
                std::process::exit(1);
            }) = value;
    }

    pub(crate) fn set_u32(&mut self, at: usize, value: u32) {
        self.buf
            .get_mut(at..at + 4)
            .context("out of bounds")
            .unwrap_or_else(|err| {
                eprintln!("{err:?}");
                std::process::exit(1);
            })
            .copy_from_slice(&value.to_le_bytes());
    }
}
