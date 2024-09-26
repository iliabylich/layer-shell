pub(crate) trait Singleton: Sized {
    fn get() -> &'static mut Self;
    fn set(v: Self);
    #[allow(dead_code)]
    fn is_set() -> bool;
}

macro_rules! singleton {
    ($t:tt) => {
        use $crate::utils::Singleton;

        static mut INSTANCE: Option<$t> = None;

        impl Singleton for $t {
            fn get() -> &'static mut Self {
                unsafe { INSTANCE.as_mut().unwrap() }
            }

            fn set(v: Self) {
                unsafe { INSTANCE = Some(v) }
            }

            fn is_set() -> bool {
                unsafe { INSTANCE.as_ref().is_some() }
            }
        }

        #[allow(dead_code)]
        fn this() -> &'static mut $t {
            $t::get()
        }
    };
}

pub(crate) use singleton;
