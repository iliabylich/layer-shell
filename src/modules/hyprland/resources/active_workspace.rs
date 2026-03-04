use crate::modules::hyprland::resources::{WriterReply, WriterResource};
use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::borrow::Cow;

pub(crate) struct ActiveWorkspaceResource;
impl WriterResource for ActiveWorkspaceResource {
    fn command(&self) -> Cow<'static, str> {
        Cow::Borrowed("[[BATCH]]j/activeworkspace")
    }

    fn parse(&self, json: &str) -> Result<WriterReply> {
        #[derive(Deserialize)]
        struct Workspace {
            id: u64,
        }
        let workspace: Workspace =
            serde_json::from_str(json).context("malformed activeworkspace response")?;
        Ok(WriterReply::ActiveWorkspace(workspace.id))
    }
}
