macro_rules! singleton {
    ($t:tt) => {
        static mut INSTANCE: Option<$t> = None;

        impl $t {
            fn get() -> &'static mut Self {
                unsafe { INSTANCE.as_mut().unwrap() }
            }

            fn set(v: Self) {
                unsafe { INSTANCE = Some(v) }
            }
        }
    };
}

pub(crate) use singleton;
