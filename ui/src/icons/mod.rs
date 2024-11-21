// #[allow(non_snake_case)]
mod gen;
pub(crate) use gen::*;

pub(crate) fn load() {
    unsafe { init_icons() }
}
