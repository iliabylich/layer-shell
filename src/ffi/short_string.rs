use crate::macros::report_and_exit;

pub const MAX_LEN: usize = 128;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShortString {
    pub(crate) bytes: [u8; MAX_LEN],
}

const _: () = {
    assert!(core::mem::size_of::<ShortString>() == 128);
};

impl From<&str> for ShortString {
    fn from(s: &str) -> Self {
        if s.len() >= MAX_LEN - 1 {
            report_and_exit!("&str is too long for ShortString: {s:?}");
        }

        let mut bytes = [0; MAX_LEN];
        bytes[..s.len()].copy_from_slice(s.as_bytes());
        bytes[s.len()] = 0;

        Self { bytes }
    }
}

impl ShortString {
    pub(crate) const fn new_const(s: &str) -> Self {
        assert!(s.len() < MAX_LEN, "too long for ShortString");

        let mut bytes = [0; MAX_LEN];
        let mut idx = 0;
        while idx < s.len() {
            bytes[idx] = s.as_bytes()[idx];
            idx += 1;
        }
        bytes[s.len()] = 0;

        Self { bytes }
    }

    pub(crate) fn len(&self) -> usize {
        self.bytes
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or_else(|| {
                report_and_exit!("ShortString is not NULL terminated: {:?}", self.bytes)
            })
    }

    pub(crate) fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.len()])
            .unwrap_or_else(|err| report_and_exit!("malformed ShortString: {err:?}"))
    }
}

impl std::fmt::Debug for ShortString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl std::fmt::Display for ShortString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl PartialEq<&str> for ShortString {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<ShortString> for &str {
    fn eq(&self, other: &ShortString) -> bool {
        self == &other.as_str()
    }
}
