macro_rules! global {
    ($name:ident, $t:ty) => {
        #[allow(non_camel_case_types)]
        struct $name;

        paste::paste! {
            #[allow(non_upper_case_globals)]
            static mut [< $name Instance >]: Option<$t> = None;

            impl $name {
                fn get() -> &'static mut $t {
                    unsafe { [< $name Instance >].as_mut().unwrap() }
                }

                fn set(v: $t) {
                    unsafe { [< $name Instance >] = Some(v) }
                }
            }
        }
    };
}
pub(crate) use global;
