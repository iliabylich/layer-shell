use anyhow::{Context as _, Result};

#[derive(Debug)]
pub(crate) enum HyprlandEvent {
    CreateWorkspace(u64),
    DestroyWorkspace(u64),
    Workspace(u64),
    LanguageChanged(String),
}

impl HyprlandEvent {
    pub(crate) fn try_parse(line: &str) -> Result<Option<Self>> {
        let (event, payload) = line.split_once(">>").with_context(|| {
            format!("malformed line from Hyprland reader socket: {line:?} (expected >> separator)")
        })?;

        let num_payload = || {
            payload
                .parse::<u64>()
                .with_context(|| format!("non-numeric payload of {event} event: {payload:?}"))
        };

        let last_substring = || {
            payload.split(",").last().with_context(|| {
                format!("expected comma separator in the payload of {event}, got {payload:?}")
            })
        };

        let event = match event {
            "createworkspace" => Self::CreateWorkspace(num_payload()?),
            "destroyworkspace" => Self::DestroyWorkspace(num_payload()?),
            "workspace" => Self::Workspace(num_payload()?),
            "activelayout" => Self::LanguageChanged(last_substring()?.to_string()),
            _ => return Ok(None),
        };

        Ok(Some(event))
    }
}
