use crate::{
    modules::hyprland::{resources::WriterResource, state::HyprlandDiff},
    utils::StringRef,
};
use anyhow::{Context as _, Result};
use serde::Deserialize;

pub(crate) struct ActiveWorkspaceResource;
impl WriterResource for ActiveWorkspaceResource {
    fn command(&self) -> StringRef {
        StringRef::new("[[BATCH]]j/activeworkspace")
    }

    fn parse(&self, json: &str) -> Result<Option<HyprlandDiff>> {
        #[derive(Deserialize)]
        struct Workspace {
            id: u64,
        }
        let workspace: Workspace =
            serde_json::from_str(json).context("malformed activeworkspace response")?;
        Ok(Some(HyprlandDiff::SetActiveWorkspaceId(workspace.id)))
    }
}
