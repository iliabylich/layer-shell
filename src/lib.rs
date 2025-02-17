#![expect(clippy::type_complexity)]
#![expect(clippy::upper_case_acronyms)]
#![expect(clippy::missing_safety_doc)]

mod channel;
mod command;
mod dbus;
mod epoll;
mod event;
mod ffi;
mod hyprctl;
mod logger;
mod macros;
mod modules;
mod timer;

use anyhow::{bail, Context as _, Result};
use channel::{CommandsChannel, EventsChannel, VerboseSender};
pub use command::*;
pub use event::Event;
use macros::fatal;

type Subscriptions = Vec<(
    extern "C" fn(*const Event, *mut std::ffi::c_void),
    *mut std::ffi::c_void,
)>;

pub(crate) struct Ctx {
    pub(crate) subscriptions: Subscriptions,
    pub(crate) events: EventsChannel,
    pub(crate) commands: CommandsChannel,
}
impl Ctx {
    pub(crate) fn from_raw(ctx: *mut std::ffi::c_void) -> &'static mut Self {
        let ctx = unsafe { ctx.cast::<Self>().as_mut() };
        ctx.unwrap_or_else(|| fatal!("Can't read NULL ctx"))
    }
}

#[no_mangle]
pub extern "C" fn layer_shell_io_init() -> *mut std::ffi::c_void {
    let logger = Box::leak(Box::new(logger::StdErrLogger::new()));
    if let Err(err) = log::set_logger(logger) {
        eprintln!("Failed to set logger: {:?}", err);
    } else {
        log::set_max_level(log::LevelFilter::Trace);
    }

    let ctx = Ctx {
        subscriptions: vec![],
        commands: CommandsChannel::new(),
        events: EventsChannel::new(),
    };
    (Box::leak(Box::new(ctx)) as *mut Ctx).cast()
}

#[no_mangle]
pub extern "C" fn layer_shell_io_subscribe(
    f: extern "C" fn(*const Event, *mut std::ffi::c_void),
    data: *mut std::ffi::c_void,
    ctx: *mut std::ffi::c_void,
) {
    Ctx::from_raw(ctx).subscriptions.push((f, data));
}

#[no_mangle]
pub extern "C" fn layer_shell_io_spawn_thread(ctx: *mut std::ffi::c_void) {
    struct SendPtr {
        ptr: *mut std::ffi::c_void,
    }
    unsafe impl Send for SendPtr {}
    impl SendPtr {
        fn get(&self) -> *mut std::ffi::c_void {
            self.ptr
        }
    }
    let ctx = SendPtr { ptr: ctx };

    std::thread::spawn(move || {
        let ctx: *mut std::ffi::c_void = ctx.get();

        if let Err(err) = layer_shell_io_run_in_place(ctx) {
            log::error!("IO thread has crashed: {:?}", err);
        }
    });
}

pub fn layer_shell_io_run_in_place(ctx: *mut std::ffi::c_void) -> Result<()> {
    use crate::epoll::{Epoll, FdId};
    use crate::modules::{
        control::Control, cpu::CPU, hyprland::Hyprland, launcher::Launcher, memory::Memory,
        network::Network, pipewire::Pipewire, session::Session, time::Time, tray::Tray,
        weather::Weather,
    };
    use crate::timer::Timer;

    let ctx = Ctx::from_raw(ctx);
    let tx = ctx.events.take_tx();
    let mut rx = ctx.commands.take_rx();

    let mut epoll = Epoll::new().context("failed to init epoll")?;
    epoll
        .add_reader_fd(rx.fd(), FdId::Command)
        .context("failed to register command handler")?;

    let mut hyprland = Hyprland::new(tx.clone());
    if let Some(fd) = hyprland.fd() {
        epoll
            .add_reader_fd(fd, FdId::HyprlandSocket)
            .context("failed to add hyprland fd to epoll")?;
    }

    let mut timer = Timer::new(1);
    if let Some(fd) = timer.fd() {
        epoll
            .add_reader_fd(fd, FdId::Timer)
            .context("failed to add timer fd to epoll")?;
    }

    let mut control = Control::new(tx.clone());
    if let Some(fd) = control.fd() {
        epoll
            .add_reader_fd(fd, FdId::ControlDBus)
            .context("failed to add control dbus fd to epoll")?;
    }

    let mut pipewire = Pipewire::new(tx.clone());
    if let Some(fd) = pipewire.fd() {
        epoll
            .add_reader_fd(fd, FdId::PipewireDBus)
            .context("failed to add pipewire dbus fd to epoll")?;
    }

    let mut network = Network::new(tx.clone());
    if let Some(fd) = network.fd() {
        epoll
            .add_reader_fd(fd, FdId::NetworkDBus)
            .context("failed to add network dbus fd to epoll")?;
    }

    let mut launcher = Launcher::new(tx.clone());
    if let Some(global_inotify_fd) = launcher.global_inotify_fd() {
        epoll
            .add_reader_fd(global_inotify_fd, FdId::LauncherGlobalDirInotify)
            .context("failed to add launcher->global_inotify_fd to epoll")?;
    }
    if let Some(user_inotify_fd) = launcher.user_inotify_fd() {
        epoll
            .add_reader_fd(user_inotify_fd, FdId::LauncherUserDirInotify)
            .context("failed to add launcher->local_inotify_fd to epoll")?;
    }

    let mut tray = Tray::new(tx.clone());
    if let Some(fd) = tray.fd() {
        epoll
            .add_reader_fd(fd, FdId::TrayDBus)
            .context("failed to add tray dbus fd to epoll")?;
    }

    let mut weather = Weather::new(tx.clone());
    let mut memory = Memory::new(tx.clone());
    let mut time = Time::new(tx.clone());
    let mut cpu = CPU::new(tx.clone());
    let mut session = Session::new();

    loop {
        let events = epoll.poll()?;
        for event in events {
            let id = FdId::try_from(event.u64)?;
            match id {
                FdId::Timer => {
                    timer.read();
                    if timer.is_multiple_of(Time::INTERVAL) {
                        time.tick();
                    }
                    if timer.is_multiple_of(Memory::INTERVAL) {
                        memory.tick();
                    }
                    if timer.is_multiple_of(CPU::INTERVAL) {
                        cpu.tick();
                    }
                    if timer.is_multiple_of(Weather::INTERVAL) {
                        weather.tick();
                    }
                }
                FdId::HyprlandSocket => {
                    hyprland.read();
                }
                FdId::ControlDBus => {
                    control.read();
                }
                FdId::PipewireDBus => {
                    pipewire.read();
                }
                FdId::LauncherGlobalDirInotify => {
                    launcher.read_global();
                }
                FdId::LauncherUserDirInotify => {
                    launcher.read_user();
                }
                FdId::NetworkDBus => {
                    network.read();
                }
                FdId::TrayDBus => {
                    tray.read();
                }
                FdId::Command => {
                    rx.consume_signal();
                    while let Some(cmd) = rx.recv() {
                        match cmd {
                            Command::HyprlandGoToWorkspace { idx } => hyprland.go_to_workspace(idx),
                            Command::LauncherReset => launcher.reset(),
                            Command::LauncherGoUp => launcher.go_up(),
                            Command::LauncherGoDown => launcher.go_down(),
                            Command::LauncherSetSearch { search } => launcher.set_search(search),
                            Command::LauncherExecSelected => launcher.exec_selected(),
                            Command::Lock => session.lock(),
                            Command::Reboot => session.reboot(),
                            Command::Shutdown => session.shutdown(),
                            Command::Logout => session.logout(),
                            Command::TriggerTray { uuid } => tray.trigger(uuid),
                            Command::SpawnNetworkEditor => network.spawn_network_editor(),
                            Command::SpawnSystemMonitor => memory.spawn_system_monitor(),
                        }
                    }
                }
                FdId::Last => bail!("got fd id = Last (bug?)"),
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn layer_shell_io_poll_events(ctx: *mut std::ffi::c_void) {
    let ctx = Ctx::from_raw(ctx);
    while let Some(event) = ctx.events.rx.recv() {
        log::info!("Received event {:?}", event);

        for (sub, data) in ctx.subscriptions.iter() {
            sub(&event, *data);
        }
    }
}
