#[repr(C)]
pub enum COption<T> {
    None,
    Some(T),
}

impl<T> From<Option<T>> for COption<T> {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(value) => Self::Some(value),
            None => Self::None,
        }
    }
}

impl<T> std::fmt::Debug for COption<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Some(value) => f.debug_tuple("Some").field(value).finish(),
        }
    }
}
