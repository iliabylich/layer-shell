use anyhow::{Context as _, Result};

pub(crate) struct UUID;

impl UUID {
    pub(crate) fn encode(service: &str, menu: &str, id: i32) -> String {
        format!("{}**{}**{}", service, menu, id)
    }

    pub(crate) fn decode(uuid: &str) -> Result<(String, String, i32)> {
        let (service, rest) = uuid
            .split_once("**")
            .with_context(|| format!("expected at least one || in {:?}", uuid))?;
        let (menu, id) = rest
            .split_once("**")
            .with_context(|| format!("expected at least two || separators in {:?}", uuid))?;

        let id = id
            .parse::<i32>()
            .with_context(|| format!("ID (the last part) is not a i32 in {:?}", uuid))?;

        Ok((service.to_string(), menu.to_string(), id))
    }
}
