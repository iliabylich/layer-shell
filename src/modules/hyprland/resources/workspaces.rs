use crate::modules::hyprland::resources::{WriterReply, WriterResource};
use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::borrow::Cow;

pub(crate) struct WorkspacesResource;

impl WriterResource for WorkspacesResource {
    fn command(&self) -> Cow<'static, str> {
        Cow::Borrowed("[[BATCH]]j/workspaces")
    }

    fn parse(&self, json: &str) -> Result<WriterReply> {
        #[derive(Debug, Deserialize)]
        struct Workspace {
            id: u64,
        }
        let workspaces: Vec<Workspace> =
            serde_json::from_str(json).context("malformed workspaces response")?;

        let workspace_ids = workspaces.into_iter().map(|w| w.id).collect();

        Ok(WriterReply::WorkspaceList(workspace_ids))
    }
}
