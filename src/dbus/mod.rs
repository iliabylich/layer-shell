#[expect(clippy::needless_lifetimes)]
mod gen;

pub(crate) mod nm;

pub(crate) use gen::nm::OrgFreedesktopNetworkManagerStateChanged;
