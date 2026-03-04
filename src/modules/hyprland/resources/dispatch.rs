use crate::modules::hyprland::resources::{WriterReply, WriterResource};
use anyhow::Result;
use std::borrow::Cow;

pub(crate) struct DispatchResource {
    cmd: String,
}
impl DispatchResource {
    pub(crate) fn new(cmd: String) -> Self {
        Self { cmd }
    }
}
impl WriterResource for DispatchResource {
    fn command(&self) -> Cow<'static, str> {
        Cow::Owned(format!("dispatch {}", self.cmd))
    }

    fn parse(&self, reply: &str) -> Result<WriterReply> {
        if reply != "ok" {
            log::error!("invalid response from hyprctl dispatch: expected 'ok', got {reply:?}");
        }
        Ok(WriterReply::None)
    }
}
