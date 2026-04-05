use crate::utils::report_and_exit;

const SLOTS_COUNT: usize = 100;

#[derive(Debug)]
struct StringPool {
    slots: [Slot; SLOTS_COUNT],
}

static mut STRING_POOL: StringPool = StringPool::new();

impl StringPool {
    const fn new() -> Self {
        Self {
            slots: [Slot::empty(); SLOTS_COUNT],
        }
    }

    fn _alloc(&mut self, s: &str) -> StringRef {
        let Some(slot) = self.slots.iter_mut().find(|slot| slot.free) else {
            for (idx, slot) in self.slots.iter().enumerate() {
                log::error!("slot {idx}: {:?}", slot.as_str());
            }
            report_and_exit!("not enough space in StringPool");
        };
        slot.acquire(s);
        StringRef { slot }
    }
}

const MAX_STRING_LEN: usize = 256;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct Slot {
    s: [u8; MAX_STRING_LEN],
    len: usize,
    refcount: u8,
    free: bool,
}

impl Slot {
    const fn empty() -> Self {
        Self {
            s: [0; MAX_STRING_LEN],
            len: 0,
            refcount: 0,
            free: true,
        }
    }

    fn acquire(&mut self, s: &str) {
        if !self.free {
            report_and_exit!("bug: Slot in string pool is not free");
        }

        if s.len() >= MAX_STRING_LEN {
            report_and_exit!(
                "string is too long to be in a StringPool (len is {})",
                s.len()
            );
        }

        let bytes = s.as_bytes();
        self.s[..bytes.len()].copy_from_slice(bytes);
        self.len = s.len();
        self.free = false;
        self.refcount = 1;
    }

    fn release(&mut self) {
        self.s = [0; 256];
        self.len = 0;
        self.refcount = 0;
        self.free = true;
    }

    fn as_bytes(&self) -> &[u8] {
        &self.s[..self.len]
    }

    fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(self.as_bytes()) }
    }

    fn addref(&mut self) {
        self.refcount += 1;
    }

    fn delref(&mut self) {
        self.refcount -= 1;
    }
}

pub struct StringRef {
    slot: *mut Slot,
}

impl StringRef {
    pub(crate) fn new(s: &str) -> Self {
        #[expect(static_mut_refs)]
        unsafe {
            STRING_POOL._alloc(s)
        }
    }

    #[expect(clippy::mut_from_ref)]
    fn slot(&self) -> &mut Slot {
        unsafe { &mut *self.slot }
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.slot().as_bytes()
    }

    pub(crate) fn as_str(&self) -> &str {
        self.slot().as_str()
    }

    pub(crate) fn len(&self) -> usize {
        self.as_str().len()
    }
}

impl core::fmt::Debug for StringRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl core::fmt::Display for StringRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl PartialEq for StringRef {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<&str> for StringRef {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<StringRef> for &str {
    fn eq(&self, other: &StringRef) -> bool {
        *self == other.as_str()
    }
}

impl core::hash::Hash for StringRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl Eq for StringRef {}

impl Default for StringRef {
    fn default() -> Self {
        Self::new("")
    }
}

impl Clone for StringRef {
    fn clone(&self) -> Self {
        self.slot().addref();
        Self { slot: self.slot }
    }
}

impl Drop for StringRef {
    fn drop(&mut self) {
        let slot = self.slot();

        slot.delref();
        if slot.refcount == 0 {
            slot.release();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_pool() {
        let mut pool = StringPool::new();

        let s1 = pool._alloc("foo");
        assert!(!pool.slots[0].free);

        let s2 = pool._alloc("bar");
        assert!(!pool.slots[1].free);

        drop(s2);
        assert!(pool.slots[1].free);

        drop(s1);
        assert!(pool.slots[0].free);
    }

    #[test]
    fn test_string_pool_reuse() {
        let mut pool = StringPool::new();

        for _ in 0..2 {
            let strings = (0..SLOTS_COUNT)
                .map(|_| pool._alloc("foo"))
                .collect::<Vec<_>>();

            for idx in 0..SLOTS_COUNT {
                assert!(
                    !pool.slots[idx].free,
                    "expected slot at {idx} to be occupied"
                );
            }

            drop(strings);
        }
    }
}
