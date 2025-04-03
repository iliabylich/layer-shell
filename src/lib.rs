#![expect(clippy::type_complexity)]
#![expect(clippy::upper_case_acronyms)]
#![expect(clippy::not_unsafe_ptr_arg_deref)]

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

use anyhow::{Context as _, Result, bail};
use channel::{CommandsChannel, EventsChannel, VerboseSender};
pub use command::*;
pub use event::Event;
use macros::fatal;

type Subscriptions = Vec<(
    extern "C" fn(*const Event, *mut std::ffi::c_void),
    *mut std::ffi::c_void,
)>;

pub struct Ctx {
    subscriptions: Subscriptions,
    events: EventsChannel,
    commands: CommandsChannel,
}
impl Ctx {
    pub(crate) fn from_raw(ctx: *mut Self) -> &'static mut Self {
        let ctx = unsafe { ctx.as_mut() };
        ctx.unwrap_or_else(|| fatal!("Can't read NULL ctx"))
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn io_init() -> *mut Ctx {
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
    Box::leak(Box::new(ctx))
}

#[unsafe(no_mangle)]
pub extern "C" fn io_subscribe(
    f: extern "C" fn(*const Event, *mut std::ffi::c_void),
    data: *mut std::ffi::c_void,
    ctx: *mut Ctx,
) {
    Ctx::from_raw(ctx).subscriptions.push((f, data));
}

#[unsafe(no_mangle)]
pub extern "C" fn io_spawn_thread(ctx: *mut Ctx) {
    struct SendPtr {
        ptr: *mut Ctx,
    }
    unsafe impl Send for SendPtr {}
    impl SendPtr {
        fn get(&self) -> *mut Ctx {
            self.ptr
        }
    }
    let ctx = SendPtr { ptr: ctx };

    std::thread::spawn(move || {
        let ctx: *mut Ctx = ctx.get();

        if let Err(err) = io_run_in_place(ctx) {
            log::error!("IO thread has crashed: {:?}", err);
        }
    });
}

pub fn io_run_in_place(ctx: *mut Ctx) -> Result<()> {
    use crate::epoll::{Epoll, FdId};
    use crate::modules::{
        control::Control, cpu::CPU, hyprland::Hyprland, launcher::Launcher, memory::Memory,
        network::Network, pipewire::Pipewire, session::Session, time::Time, tray::Tray,
        weather::Weather,
    };
    use crate::timer::Timer;

    let ctx = Ctx::from_raw(ctx);
    let tx = ctx.events.take_tx();

    let mut epoll = Epoll::new().context("failed to init epoll")?;

    let mut rx = ctx.commands.take_rx();
    epoll.add_reader(&rx)?;

    let mut hyprland = Hyprland::new(tx.clone());
    epoll.add_reader(&hyprland)?;

    let mut timer = Timer::new(1);
    epoll.add_reader(&timer)?;

    let mut control = Control::new(tx.clone());
    epoll.add_reader(&control)?;

    let mut pipewire = Pipewire::new(tx.clone());
    epoll.add_reader(&pipewire)?;

    let mut network = Network::new(tx.clone());
    epoll.add_reader(&network)?;

    let (mut launcher, mut global_dir_inotify, mut user_dir_inotify) = Launcher::new(tx.clone());
    epoll.add_reader(&global_dir_inotify)?;
    epoll.add_reader(&user_dir_inotify)?;

    let mut tray = Tray::new(tx.clone());
    epoll.add_reader(&tray)?;

    let mut weather = Weather::new(tx.clone());
    let mut memory = Memory::new(tx.clone());
    let mut time = Time::new(tx.clone());
    let mut cpu = CPU::new(tx.clone());
    let mut session = Session::new();

    let mut events = Vec::with_capacity(100);

    loop {
        epoll.poll(&mut events)?;
        for event in events.iter() {
            let id = FdId::try_from(event.u64)?;
            match id {
                FdId::Timer => {
                    let Some(ticks) = epoll.read_from_or_disable(&mut timer) else {
                        continue;
                    };
                    if ticks.is_multiple_of(Time::INTERVAL) {
                        time.tick();
                    }
                    if ticks.is_multiple_of(Memory::INTERVAL) {
                        memory.tick();
                    }
                    if ticks.is_multiple_of(CPU::INTERVAL) {
                        cpu.tick();
                    }
                    if ticks.is_multiple_of(Weather::INTERVAL) {
                        weather.tick();
                    }
                }
                FdId::HyprlandSocket => {
                    epoll.read_from_or_disable(&mut hyprland);
                }
                FdId::ControlDBus => {
                    epoll.read_from_or_disable(&mut control);
                }
                FdId::PipewireDBus => {
                    epoll.read_from_or_disable(&mut pipewire);
                }
                FdId::LauncherGlobalDirInotify => {
                    epoll.read_from_or_disable(&mut global_dir_inotify);
                }
                FdId::LauncherUserDirInotify => {
                    epoll.read_from_or_disable(&mut user_dir_inotify);
                }
                FdId::NetworkDBus => {
                    epoll.read_from_or_disable(&mut network);
                }
                FdId::TrayDBus => {
                    epoll.read_from_or_ignore(&mut tray);
                }
                FdId::Command => {
                    rx.consume_signal();
                    while let Some(cmd) = rx.recv() {
                        match cmd {
                            Command::HyprlandGoToWorkspace { idx } => {
                                hyprland.with(|hyprland| hyprland.go_to_workspace(idx))
                            }
                            Command::LauncherReset => launcher.reset(),
                            Command::LauncherGoUp => launcher.go_up(),
                            Command::LauncherGoDown => launcher.go_down(),
                            Command::LauncherSetSearch { search } => launcher.set_search(search),
                            Command::LauncherExecSelected => launcher.exec_selected(),
                            Command::Lock => session.lock(),
                            Command::Reboot => session.reboot(),
                            Command::Shutdown => session.shutdown(),
                            Command::Logout => session.logout(),
                            Command::ChangeTheme => session.change_theme(),
                            Command::TriggerTray { uuid } => tray.with(|tray| tray.trigger(uuid)),
                            Command::SpawnNetworkEditor => {
                                network.with(|network| network.spawn_network_editor())
                            }
                            Command::SpawnSystemMonitor => memory.spawn_system_monitor(),
                        }
                    }
                }
                FdId::Disconnected => bail!("got fd id = Last (bug?)"),
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn io_poll_events(ctx: *mut Ctx) {
    let ctx = Ctx::from_raw(ctx);
    while let Some(event) = ctx.events.rx.recv() {
        log::info!("Received event {:?}", event);

        for (sub, data) in ctx.subscriptions.iter() {
            sub(&event, *data);
        }
    }
}
