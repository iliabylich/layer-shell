#[derive(Debug, Clone, Copy)]
pub(crate) struct Tick(pub(crate) u64);

impl Tick {
    pub(crate) fn is_multiple_of(self, n: u64) -> bool {
        self.0 % n == 0
    }
}
