use crate::modules::hyprland::state::HyprlandDiff;
use anyhow::Result;
use std::borrow::Cow;

mod active_workspace;
mod caps_lock;
mod devices;
mod dispatch;
mod workspaces;

pub(crate) use active_workspace::ActiveWorkspaceResource;
pub(crate) use caps_lock::CapsLockResource;
pub(crate) use devices::DevicesResource;
pub(crate) use dispatch::DispatchResource;
pub(crate) use workspaces::WorkspacesResource;

pub(crate) trait WriterResource {
    fn command(&self) -> Cow<'static, str>;
    fn parse(&self, json: &str) -> Result<Option<HyprlandDiff>>;
}
