use crate::{
    modules::hyprland::{resources::WriterResource, state::HyprlandDiff},
    utils::{ArrayWriter, StringRef, report_and_exit},
};
use anyhow::Result;
use core::fmt::Write;

pub(crate) struct DispatchResource {
    cmd: String,
}
impl DispatchResource {
    pub(crate) fn new(cmd: String) -> Self {
        Self { cmd }
    }
}
impl WriterResource for DispatchResource {
    fn command(&self) -> StringRef {
        let mut buf = [0; 128];
        let mut writer = ArrayWriter::new(&mut buf);
        write!(&mut writer, "dispatch {}", self.cmd).unwrap_or_else(|err: core::fmt::Error| {
            report_and_exit!("failed to write command to buffer: {err:?}")
        });
        StringRef::new(
            writer.as_str().unwrap_or_else(|err| {
                report_and_exit!("command is too long for StringRef: {err:?}")
            }),
        )
    }

    fn parse(&self, reply: &str) -> Result<Option<HyprlandDiff>> {
        if reply != "ok" {
            log::error!("invalid response from hyprctl dispatch: expected 'ok', got {reply:?}");
        }
        Ok(None)
    }
}
