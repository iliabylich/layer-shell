use anyhow::{Context, Result};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FixedSizeArrray<const N: usize, T> {
    items: [T; N],
    count: usize,
}

impl<const N: usize, T> FixedSizeArrray<N, T> {
    pub(crate) fn new() -> Self
    where
        T: Default + Copy,
    {
        Self {
            items: [T::default(); _],
            count: 0,
        }
    }

    pub(crate) const fn filled(item: T, count: usize) -> Self
    where
        T: Copy,
    {
        Self {
            items: [item; _],
            count,
        }
    }

    pub(crate) fn empty_with_default_fn<F>(f: F) -> Self
    where
        F: Fn() -> T,
    {
        let items: [T; N] = core::array::from_fn(|_idx| f());
        Self { items, count: 0 }
    }

    pub(crate) fn push(&mut self, item: T) -> Result<()> {
        let slot = self
            .items
            .get_mut(self.count)
            .context("fixed sized array overlow")?;
        *slot = item;
        self.count = self
            .count
            .checked_add(1)
            .context("too many items in fixed size array")?;
        Ok(())
    }

    pub(crate) const fn len(&self) -> usize {
        self.count
    }

    pub(crate) fn get(&self, idx: usize) -> Option<&T> {
        if idx < self.count {
            self.items.get(idx)
        } else {
            None
        }
    }

    pub(crate) const fn as_ptr(&self) -> *const T {
        self.items.as_ptr()
    }
}

impl<const N: usize, T> core::fmt::Debug for FixedSizeArrray<N, T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[")?;
        for (idx, item) in self.items.iter().take(self.count).enumerate() {
            if idx != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{item:?}")?;
        }
        write!(f, "]")?;
        Ok(())
    }
}
