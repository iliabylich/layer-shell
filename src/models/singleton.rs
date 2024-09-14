macro_rules! singleton {
    ($t:tt, $name:ident) => {
        static mut $name: Option<$t> = None;

        impl $t {
            fn get() -> &'static mut Self {
                unsafe { $name.as_mut().unwrap() }
            }

            fn set(v: Self) {
                unsafe { $name = Some(v) }
            }

            #[allow(dead_code)]
            fn is_set() -> bool {
                unsafe { $name.as_ref().is_some() }
            }
        }
    };

    ($t:tt) => {
        singleton!($t, INSTANCE);
    };
}

pub(crate) use singleton;
