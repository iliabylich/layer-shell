#[repr(C)]
#[expect(unused)]
pub enum FFIOption<T> {
    None,
    Some(T),
}

impl<T, U> From<Option<T>> for FFIOption<U>
where
    U: From<T>,
{
    fn from(v: Option<T>) -> Self {
        match v {
            Some(value) => Self::Some(value.into()),
            None => Self::None,
        }
    }
}

impl<T> std::fmt::Debug for FFIOption<T>
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
