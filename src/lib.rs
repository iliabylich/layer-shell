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
mod timer;

use anyhow::Result;
use channel::{CommandReceiver, CommandSender, EventReceiver, EventSender};
use command::*;
use event::Event;
use r#loop::Loop;
use macros::fatal;
use pyo3::{
    Bound, PyResult, pyclass, pymethods, pymodule,
    types::{PyModule, PyModuleMethods as _},
};

pub struct IoCtx {
    tx: EventSender,
    rx: CommandReceiver,
}
#[pyclass]
pub struct UiCtx {
    tx: EventReceiver,
    rx: CommandSender,
}

#[pyclass]
#[derive(Default)]
pub struct MaybeIoCtx(Option<IoCtx>);

#[pyclass]
pub struct IO;

impl IO {
    pub fn run_in_place(io_ctx: MaybeIoCtx) -> Result<()> {
        if let Some(io_ctx) = io_ctx.0 {
            let tx = io_ctx.tx;
            let rx = io_ctx.rx;

            let r#loop = Loop::new(tx, rx)?;
            r#loop.start();
        } else {
            anyhow::bail!("Can't spawn thread, IO ctx has already been taken")
        }
    }
}

#[pymethods]
impl IO {
    #[staticmethod]
    pub fn init() -> (MaybeIoCtx, UiCtx) {
        env_logger::init();

        let (etx, erx) = channel::events();
        let (ctx, crx) = channel::commands();

        let io_ctx = IoCtx { tx: etx, rx: crx };
        let ui_ctx = UiCtx { tx: erx, rx: ctx };
        (MaybeIoCtx(Some(io_ctx)), ui_ctx)
    }
    #[staticmethod]
    fn spawn_thread(io_ctx: &mut MaybeIoCtx) {
        let io_ctx = std::mem::take(io_ctx);
        std::thread::spawn(move || {
            if let Err(err) = Self::run_in_place(io_ctx) {
                log::error!("IO thread has crashed: {:?}", err);
            }
        });
    }
    #[staticmethod]
    fn poll_events(ui_ctx: &mut UiCtx) -> Vec<Event> {
        let mut out = vec![];
        while let Some(event) = ui_ctx.tx.recv() {
            log::info!("Received event {:?}", event);
            out.push(event);
        }
        out
    }
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

#[pymodule]
fn liblayer_shell_io(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use event::{LauncherApp, LauncherAppIcon, TrayApp, TrayIcon, TrayItem, WifiStatus};
    use modules::weather::WeatherCode;

    m.add_class::<IO>()?;
    m.add_class::<MaybeIoCtx>()?;
    m.add_class::<UiCtx>()?;
    m.add_class::<Event>()?;
    m.add_class::<Commands>()?;
    m.add_class::<WifiStatus>()?;
    m.add_class::<WeatherCode>()?;
    m.add_class::<LauncherApp>()?;
    m.add_class::<LauncherAppIcon>()?;
    m.add_class::<TrayApp>()?;
    m.add_class::<TrayItem>()?;
    m.add_class::<TrayIcon>()?;

    Ok(())
}
