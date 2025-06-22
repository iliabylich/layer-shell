// #![expect(clippy::type_complexity)]
// #![expect(clippy::upper_case_acronyms)]
#![expect(static_mut_refs)]

// mod channel;
mod command;
// mod dbus;
mod event;
// mod fd_id;
// mod hyprctl;
// mod r#loop;
// mod macros;
// mod modules;
// mod poll;
// mod timer;
mod main_loop;

use anyhow::{Context as _, Result};
use command::Command;
use event::Event;
pub use ffi::{CArray, CString};
use main_loop::MainLoop;

use tokio::sync::{
    OnceCell,
    mpsc::{Receiver, Sender},
};

static mut ETX: OnceCell<Sender<Event>> = OnceCell::const_new();
static mut ERX: OnceCell<Receiver<Event>> = OnceCell::const_new();

static mut CTX: OnceCell<Sender<Command>> = OnceCell::const_new();
static mut CRX: OnceCell<Receiver<Command>> = OnceCell::const_new();

static mut THREAD_HANDLE: OnceCell<std::thread::JoinHandle<()>> = OnceCell::const_new();

/// # Safety
///
/// This function must be called at most once.
pub fn io_run_in_place() -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|err| {
            log::error!("failed to start tokio runtime: {err:?}");
            std::process::exit(1);
        });

    rt.block_on(async move {
        let Some(etx) = (unsafe { ETX.take() }) else {
            log::error!("ETX is not set, did you call io_init()?");
            std::process::exit(1);
        };
        let Some(crx) = (unsafe { CRX.take() }) else {
            log::error!("CRX is not set, did you call io_init()?");
            std::process::exit(1);
        };

        let mut main_loop = match MainLoop::new(etx, crx).await {
            Ok(main_loop) => main_loop,
            Err(err) => {
                log::error!("failed to instantiate main loop, exiting: {err:?}");
                std::process::exit(1);
            }
        };

        if let Err(err) = main_loop.start().await {
            log::error!("main loop error, stopping: {err:?}");
            std::process::exit(1);
        }
    });

    log::info!("tokio has finished");
    Ok(())
}

#[unsafe(no_mangle)]
pub extern "C" fn io_init() {
    env_logger::init();

    fn try_io_init() -> Result<()> {
        let (etx, erx) = tokio::sync::mpsc::channel::<Event>(100);
        let (ctx, crx) = tokio::sync::mpsc::channel::<Command>(100);

        unsafe {
            const ERR: &str = "io_init must be called once";
            ETX.set(etx).context(ERR)?;
            ERX.set(erx).context(ERR)?;
            CTX.set(ctx).context(ERR)?;
            CRX.set(crx).context(ERR)?;
        }

        Ok(())
    }

    if let Err(err) = try_io_init() {
        log::error!("{err:?}");
        std::process::exit(1);
    }
}
/// # Safety
///
/// This function must be called at most once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn io_spawn_thread() {
    let handle = std::thread::spawn(move || {
        if let Err(err) = io_run_in_place() {
            log::error!("IO thread has crashed: {:?}", err);
        }
    });

    unsafe {
        if THREAD_HANDLE.set(handle).is_err() {
            log::error!("io_spawn_thread must be called once");
        }
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn io_poll_events() -> CArray<Event> {
    let mut out = vec![];
    let erx = unsafe {
        ERX.get_mut().unwrap_or_else(|| {
            log::error!("ERX is not set, did you call io_init()?");
            std::process::exit(1);
        })
    };
    while let Ok(event) = erx.try_recv() {
        log::info!("Received event {:?}", event);
        out.push(event);
    }
    out.into()
}
#[unsafe(no_mangle)]
pub extern "C" fn io_drop_events(events: CArray<Event>) {
    drop(events)
}
#[unsafe(no_mangle)]
pub extern "C" fn io_finalize() {
    fn try_io_finalize() -> Result<()> {
        unsafe {
            CTX.get()
                .context("no CTX, did you call io_init()?")?
                .blocking_send(Command::FinishIoThread)
                .context("failed to send Command, channel is closed")?;
        };
        log::info!("Waiting for IO thread to finish...");
        unsafe {
            THREAD_HANDLE
                .take()
                .context("THREAD_HANDLE is not set, did you call io_init()")?
                .join()
                .map_err(|_| anyhow::anyhow!("io thread is not running, can't stop gracefully"))?;
        };
        log::info!("IO thread has finished...");
        Ok(())
    }

    if let Err(err) = try_io_finalize() {
        log::error!("{err:?}");
        std::process::exit(1);
    }
}

// #[unsafe(no_mangle)]
// pub extern "C" fn io_hyprland_go_to_workspace(ui_ctx: &mut UiCtx, idx: usize) {
//     ui_ctx.rx.send(Command::HyprlandGoToWorkspace { idx });
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_lock(ui_ctx: &mut UiCtx) {
//     ui_ctx.rx.send(Command::Lock);
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_reboot(ui_ctx: &mut UiCtx) {
//     ui_ctx.rx.send(Command::Reboot);
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_shutdown(ui_ctx: &mut UiCtx) {
//     ui_ctx.rx.send(Command::Shutdown);
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_logout(ui_ctx: &mut UiCtx) {
//     ui_ctx.rx.send(Command::Logout);
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_trigger_tray(ui_ctx: &mut UiCtx, uuid: *const u8) {
//     ui_ctx.rx.send(Command::TriggerTray {
//         uuid: CString::from(uuid).into(),
//     });
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_spawn_network_editor(ui_ctx: &mut UiCtx) {
//     ui_ctx.rx.send(Command::SpawnNetworkEditor);
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_spawn_system_monitor(ui_ctx: &mut UiCtx) {
//     ui_ctx.rx.send(Command::SpawnSystemMonitor);
// }
// #[unsafe(no_mangle)]
// pub extern "C" fn io_change_theme(ui_ctx: &mut UiCtx) {
//     ui_ctx.rx.send(Command::ChangeTheme);
// }
