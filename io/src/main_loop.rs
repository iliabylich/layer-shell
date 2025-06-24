use crate::{command::Command, event::Event};
use anyhow::{Result, anyhow, bail};
use clock::Clock;
use control::Control;
use cpu::CPU;
use hyprland::Hyprland;
use memory::Memory;
use network::Network;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub(crate) struct MainLoop {
    etx: UnboundedSender<Event>,
    crx: UnboundedReceiver<Command>,

    hyprland: Hyprland,
    cpu: CPU,
    memory: Memory,
    clock: Clock,
    control: Control,
    network: Network,
}

impl MainLoop {
    pub(crate) async fn new(
        etx: UnboundedSender<Event>,
        crx: UnboundedReceiver<Command>,
    ) -> Result<Self> {
        let hyprland = Hyprland::start();
        let cpu = CPU::start();
        let memory = Memory::start();
        let clock = Clock::start();
        let control = Control::start();
        let network = Network::start();

        Ok(Self {
            etx,
            crx,
            hyprland,
            cpu,
            memory,
            clock,
            control,
            network,
        })
    }

    pub(crate) async fn start(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                Some(e) = self.hyprland.recv() => {
                    self.emit("Hyprland", e).await?;
                }

                Some(e) = self.cpu.recv() => {
                    self.emit("CPU", e).await?;
                }

                Some(e) = self.memory.recv() => {
                    self.emit("Memory", e).await?;
                }

                Some(e) = self.clock.recv() => {
                    self.emit("Clock", e).await?;
                }

                Some(e) = self.control.recv() => {
                    self.emit("Control", e).await?;
                }

                Some(e) = self.network.recv() => {
                    self.emit("Network", e).await?;
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
            .map_err(|_| anyhow!("failed to emit Event, channel is closed"))
    }

    async fn hyprctl_dispatch(&mut self, cmd: impl AsRef<str>) {
        if let Err(err) = self.hyprland.hyprctl_dispatch(cmd).await {
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
