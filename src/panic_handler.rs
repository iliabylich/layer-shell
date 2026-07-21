#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    log::error!("Panic: {}", info.message());
    if let Some(location) = info.location() {
        log::error!("    at {}", location);
    }
    unsafe { libc::_exit(1) }
}

#[unsafe(no_mangle)]
extern "C" fn rust_eh_personality() {}
