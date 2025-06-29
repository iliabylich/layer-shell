mod command;
mod event;
mod main_loop;

use anyhow::{Context as _, Result};
use command::Command;
use event::Event;
pub use ffi::{CArray, CString};
use main_loop::MainLoop;
use std::cell::RefCell;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

thread_local! {
    static ETX: RefCell<Option<UnboundedSender<Event>>> = const { RefCell::new(None) };
    static ERX: RefCell<Option<UnboundedReceiver<Event>>> = const { RefCell::new(None) };

    static CTX: RefCell<Option<UnboundedSender<Command>>> = const { RefCell::new(None) };
    static CRX: RefCell<Option<UnboundedReceiver<Command>>> = const { RefCell::new(None) };

    static THREAD_HANDLE: RefCell<Option<std::thread::JoinHandle<()>>> = const { RefCell::new(None) };
}

fn io_run_in_place(etx: UnboundedSender<Event>, crx: UnboundedReceiver<Command>) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|err| {
            log::error!("failed to start tokio runtime: {err:?}");
            std::process::exit(1);
        });

    rt.block_on(async move {
        let main_loop = match MainLoop::new(etx, crx).await {
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

    let (etx, erx) = tokio::sync::mpsc::unbounded_channel::<Event>();
    let (ctx, crx) = tokio::sync::mpsc::unbounded_channel::<Command>();

    ETX.set(Some(etx));
    ERX.set(Some(erx));
    CTX.set(Some(ctx));
    CRX.set(Some(crx));
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_thread() {
    let Some(etx) = ETX.take() else {
        log::error!("ETX is not set, did you call io_init()?");
        std::process::exit(1);
    };
    let Some(crx) = CRX.take() else {
        log::error!("CRX is not set, did you call io_init()?");
        std::process::exit(1);
    };

    let handle = std::thread::spawn(move || {
        if let Err(err) = io_run_in_place(etx, crx) {
            log::error!("IO thread has crashed: {:?}", err);
        }
    });

    THREAD_HANDLE.set(Some(handle));
}
#[unsafe(no_mangle)]
pub extern "C" fn io_poll_events() -> CArray<Event> {
    let mut out = vec![];

    fn try_recv() -> Result<Event> {
        ERX.with_borrow_mut(|erx| {
            let erx = erx.as_mut().unwrap_or_else(|| {
                log::error!("ERX is not set, did you call io_init()?");
                std::process::exit(1);
            });
            erx.try_recv().context("recv() failed")
        })
    }

    while let Ok(event) = try_recv() {
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
        send_command(Command::FinishIoThread);
        log::info!("Waiting for IO thread to finish...");
        THREAD_HANDLE
            .take()
            .context("THREAD_HANDLE is not set, did you call io_init()")?
            .join()
            .map_err(|_| anyhow::anyhow!("io thread is not running, can't stop gracefully"))?;
        log::info!("IO thread has finished...");
        Ok(())
    }

    if let Err(err) = try_io_finalize() {
        log::error!("{err:?}");
        std::process::exit(1);
    }
}

fn send_command(cmd: Command) {
    CTX.with_borrow_mut(|ctx| {
        let ctx = ctx.as_mut().unwrap_or_else(|| {
            log::error!("no CTX, did you call io_init()?");
            std::process::exit(1);
        });

        if ctx.send(cmd).is_err() {
            log::error!("failed to send Command, channel is closed");
            std::process::exit(1);
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn io_hyprland_go_to_workspace(workspace: usize) {
    send_command(Command::HyprlandGoToWorkspace { workspace });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_lock() {
    send_command(Command::Lock);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_reboot() {
    send_command(Command::Reboot);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_shutdown() {
    send_command(Command::Shutdown);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_logout() {
    send_command(Command::Logout);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_trigger_tray(uuid: *const std::ffi::c_char) {
    send_command(Command::TriggerTray {
        uuid: CString::from(uuid).into(),
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_wifi_editor() {
    send_command(Command::SpawnWiFiEditor);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_bluetooh_editor() {
    send_command(Command::SpawnBluetoothEditor);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_system_monitor() {
    send_command(Command::SpawnSystemMonitor);
}
#[unsafe(no_mangle)]
pub extern "C" fn io_change_theme() {
    send_command(Command::ChangeTheme);
}
