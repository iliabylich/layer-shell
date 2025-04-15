#![expect(clippy::type_complexity)]
#![expect(clippy::upper_case_acronyms)]

mod channel;
mod command;
mod dbus;
mod event;
mod fd_id;
mod hyprctl;
mod r#loop;
mod macros;
mod modules;
mod poll;
mod subscriptions;
mod timer;

use anyhow::Result;
use channel::{CommandReceiver, CommandSender, EventReceiver, EventSender};
use command::*;
use event::Event;
use r#loop::Loop;
use macros::fatal;
use subscriptions::Subscriptions;

pub struct IoCtx {
    tx: EventSender,
    rx: CommandReceiver,
}
#[pyclass]
pub struct UiCtx {
    tx: EventReceiver,
    rx: CommandSender,
    subs: Subscriptions,
}

pub fn io_init() -> (IoCtx, UiCtx) {
    env_logger::init();

    let (etx, erx) = channel::events();
    let (ctx, crx) = channel::commands();
    let subs = Subscriptions::new();

    (
        IoCtx { tx: etx, rx: crx },
        UiCtx {
            tx: erx,
            rx: ctx,
            subs,
        },
    )
}

fn io_spawn_thread(io_ctx: IoCtx) {
    std::thread::spawn(move || {
        if let Err(err) = io_run_in_place(io_ctx) {
            log::error!("IO thread has crashed: {:?}", err);
        }
    });
}

pub fn io_run_in_place(io_ctx: IoCtx) -> Result<()> {
    let tx = io_ctx.tx;
    let rx = io_ctx.rx;

    let r#loop = Loop::new(tx, rx)?;
    r#loop.start();
}

fn io_subscribe(ui_ctx: &mut UiCtx, sub: *mut PyObject) {
    ui_ctx.subs.push(sub);
}

fn io_poll_events(ui_ctx: &mut UiCtx) {
    while let Some(event) = ui_ctx.tx.recv() {
        log::info!("Received event {:?}", event);
        ui_ctx.subs.notify_each(&event);
    }
}

use pyo3::{
    Bound, PyResult,
    ffi::PyObject,
    pyclass, pyfunction, pymethods, pymodule,
    types::{PyAny, PyModule, PyModuleMethods as _},
    wrap_pyfunction,
};

#[pyclass]
pub struct MaybeIoCtx(Option<IoCtx>);

#[pyfunction]
fn init() -> (MaybeIoCtx, UiCtx) {
    let (io_ctx, ui_ctx) = io_init();
    (MaybeIoCtx(Some(io_ctx)), ui_ctx)
}
#[pyfunction]
fn spawn_thread(io_ctx: &mut MaybeIoCtx) {
    if let Some(io_ctx) = io_ctx.0.take() {
        io_spawn_thread(io_ctx);
    } else {
        eprintln!("Can't spawn thread, IO ctx has already been taken")
    }
}
#[pyfunction]
fn poll_events(ui_ctx: &mut UiCtx) {
    io_poll_events(ui_ctx);
}
#[pyfunction]
fn subscribe(ui_ctx: &mut UiCtx, sub: Bound<'_, PyAny>) {
    let sub = sub.into_ptr();
    io_subscribe(ui_ctx, sub);
}
#[pyclass]
struct Commands;
#[pymethods]
impl Commands {
    #[staticmethod]
    fn hyprland_go_to_workspace(ui_ctx: &mut UiCtx, idx: usize) {
        ui_ctx.rx.send(Command::HyprlandGoToWorkspace { idx });
    }
    #[staticmethod]
    fn launcher_reset(ui_ctx: &mut UiCtx) {
        ui_ctx.rx.send(Command::LauncherReset);
    }
    #[staticmethod]
    fn launcher_go_up(ui_ctx: &mut UiCtx) {
        ui_ctx.rx.send(Command::LauncherGoUp);
    }
    #[staticmethod]
    fn launcher_go_down(ui_ctx: &mut UiCtx) {
        ui_ctx.rx.send(Command::LauncherGoDown);
    }
    #[staticmethod]
    fn launcher_set_search(ui_ctx: &mut UiCtx, search: String) {
        ui_ctx.rx.send(Command::LauncherSetSearch { search });
    }
    #[staticmethod]
    fn launcher_exec_selected(ui_ctx: &mut UiCtx) {
        ui_ctx.rx.send(Command::LauncherExecSelected);
    }
    #[staticmethod]
    fn lock(ui_ctx: &mut UiCtx) {
        ui_ctx.rx.send(Command::Lock);
    }
    #[staticmethod]
    fn reboot(ui_ctx: &mut UiCtx) {
        ui_ctx.rx.send(Command::Reboot);
    }
    #[staticmethod]
    fn shutdown(ui_ctx: &mut UiCtx) {
        ui_ctx.rx.send(Command::Shutdown);
    }
    #[staticmethod]
    fn logout(ui_ctx: &mut UiCtx) {
        ui_ctx.rx.send(Command::Logout);
    }
    #[staticmethod]
    fn trigger_tray(ui_ctx: &mut UiCtx, uuid: String) {
        ui_ctx.rx.send(Command::TriggerTray { uuid });
    }
    #[staticmethod]
    fn spawn_network_editor(ui_ctx: &mut UiCtx) {
        ui_ctx.rx.send(Command::SpawnNetworkEditor);
    }
    #[staticmethod]
    fn spawn_system_monitor(ui_ctx: &mut UiCtx) {
        ui_ctx.rx.send(Command::SpawnSystemMonitor);
    }
    #[staticmethod]
    fn change_theme(ui_ctx: &mut UiCtx) {
        ui_ctx.rx.send(Command::ChangeTheme);
    }
}

#[pyfunction]
fn main_css() -> &'static str {
    include_str!("../main.css")
}
#[pyclass]
struct Icons;
macro_rules! icons {
    ($($name:ident,)+) => {
        #[pymethods]
        impl Icons {
            $(
                #[staticmethod]
                fn $name() -> &'static [u8] {
                    include_bytes!(concat!("../icons/", stringify!($name), ".png"))
                }
            )*

            #[staticmethod]
            fn names() -> Vec<&'static str> {
                vec![
                    $(stringify!($name),)*
                ]
            }
        }
    };
}
icons!(
    change_theme,
    download,
    foggy,
    partly_cloudy,
    power,
    question_mark,
    rainy,
    snowy,
    sunny,
    thunderstorm,
    upload,
    wifi,
);
#[pymodule]
fn liblayer_shell_io(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use event::{LauncherApp, LauncherAppIcon, TrayApp, TrayIcon, TrayItem, WifiStatus};
    use modules::weather::WeatherCode;

    m.add_class::<MaybeIoCtx>()?;
    m.add_class::<UiCtx>()?;
    m.add_class::<Event>()?;
    m.add_class::<Commands>()?;
    m.add_class::<Icons>()?;
    m.add_class::<WifiStatus>()?;
    m.add_class::<WeatherCode>()?;
    m.add_class::<LauncherApp>()?;
    m.add_class::<LauncherAppIcon>()?;
    m.add_class::<TrayApp>()?;
    m.add_class::<TrayItem>()?;
    m.add_class::<TrayIcon>()?;
    m.add_function(wrap_pyfunction!(init, m)?)?;
    m.add_function(wrap_pyfunction!(spawn_thread, m)?)?;
    m.add_function(wrap_pyfunction!(poll_events, m)?)?;
    m.add_function(wrap_pyfunction!(subscribe, m)?)?;

    m.add_function(wrap_pyfunction!(main_css, m)?)?;
    Ok(())
}
