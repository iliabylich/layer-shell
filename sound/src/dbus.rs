use zbus::proxy;

#[proxy(
    interface = "org.local.PipewireDBus",
    default_service = "org.local.PipewireDBus",
    default_path = "/org/local/PipewireDBus"
)]
pub(crate) trait PipewireDBus {
    #[zbus(property)]
    fn muted(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn volume(&self) -> zbus::Result<u32>;
}
