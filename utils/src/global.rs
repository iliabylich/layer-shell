#[macro_export]
macro_rules! global {
    ($name:ident, $t:ty) => {
        #[allow(non_camel_case_types)]
        struct $name;

        $crate::paste::paste! {
            #[allow(non_upper_case_globals)]
            static mut [< $name Instance >]: Option<$t> = None;

            impl $name {
                fn get() -> &'static mut $t {
                    unsafe {
                        match [< $name Instance >].as_mut() {
                            Some(value) => value,
                            None => {
                                eprintln!("global! {} is not set", stringify!($name));
                                std::process::exit(1);
                            }
                        }
                    }
                }

                fn set(v: $t) {
                    unsafe { [< $name Instance >] = Some(v) }
                }
            }
        }
    };
}
