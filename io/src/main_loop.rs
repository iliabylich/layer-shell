use crate::{command::Command, event::Event};
use anyhow::{Result, anyhow, bail};
use clock::Clock;
use control::Control;
use cpu::Cpu;
use futures_util::{StreamExt as _, stream::Fuse};
use hyprland::Hyprland;
use memory::Memory;
use tokio::sync::mpsc::{Receiver, Sender};

pub(crate) struct MainLoop {
    etx: Sender<Event>,
    crx: Receiver<Command>,

    hyprland: Fuse<Hyprland>,
    cpu: Fuse<Cpu>,
    memory: Fuse<Memory>,
    clock: Fuse<Clock>,
    control: Fuse<Control>,
}

impl MainLoop {
    pub(crate) async fn new(etx: Sender<Event>, crx: Receiver<Command>) -> Result<Self> {
        let hyprland = Hyprland::new().await?.fuse();
        let cpu = Cpu::new().fuse();
        let memory = Memory::new().fuse();
        let clock = Clock::new().fuse();
        let control = Control::new().await?.fuse();

        Ok(Self {
            etx,
            crx,
            hyprland,
            cpu,
            memory,
            clock,
            control,
        })
    }

    pub(crate) async fn start(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                Some(e) = self.hyprland.next() => {
                    self.emit("Hyprland", e).await?;
                }

                Some(e) = self.cpu.next() => {
                    self.emit("CPU", e).await?;
                }

                Some(e) = self.memory.next() => {
                    self.emit("Memory", e).await?;
                }

                Some(e) = self.clock.next() => {
                    self.emit("Clock", e).await?;
                }

                Some(e) = self.control.next() => {
                    self.emit("Control", e).await?;
                }

                Some(cmd) = self.crx.recv() => {
                    if matches!(cmd, Command::FinishIoThread) {
                        return Ok(());
                    }
                    self.on_command(cmd).await;
                }

                else => bail!("all streams are dead"),
            }
        }
    }

    async fn emit(&self, module: &str, e: impl Into<Event>) -> Result<()> {
        let e: Event = e.into();
        log::info!(target: module, "{e:?}");

        self.etx
            .send(e)
            .await
            .map_err(|_| anyhow!("failed to emit Event, channel is closed"))
    }

    async fn hyprctl_dispatch(&mut self, cmd: impl AsRef<str>) {
        if let Err(err) = self.hyprland.get_mut().hyprctl_dispatch(cmd).await {
            log::error!("{err:?}");
        }
    }

    async fn on_command(&mut self, cmd: Command) {
        match cmd {
            Command::FinishIoThread => unreachable!("handled by the caller"),

            Command::HyprlandGoToWorkspace { idx } => {
                self.hyprctl_dispatch(format!("workspace {}", idx + 1))
                    .await;
            }
            Command::Lock => {
                self.hyprctl_dispatch("exec hyprlock").await;
            }
            Command::Reboot => {
                self.hyprctl_dispatch("exec systemctl reboot").await;
            }
            Command::Shutdown => {
                self.hyprctl_dispatch("exec systemctl poweroff").await;
            }
            Command::Logout => {
                self.hyprctl_dispatch("exit").await;
            }
            Command::TriggerTray { uuid } => todo!(),
            Command::SpawnNetworkEditor => {
                self.hyprctl_dispatch("exec iwmenu --launcher fuzzel").await;
            }
            Command::SpawnSystemMonitor => {
                self.hyprctl_dispatch("exec gnome-system-monitor").await;
            }
            Command::ChangeTheme => {
                self.hyprctl_dispatch("exec ~/.config/hypr/wallpaper-change.sh")
                    .await
            }
        }
    }
}
