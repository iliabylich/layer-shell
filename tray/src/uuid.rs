use anyhow::{Context as _, Result};

pub(crate) struct UUID;

impl UUID {
    pub(crate) fn encode(service: &str, id: i32) -> String {
        format!("{}**{}", service, id)
    }

    pub(crate) fn decode(uuid: impl AsRef<str>) -> Result<(String, i32)> {
        let uuid = uuid.as_ref();

        let (service, id) = uuid
            .split_once("**")
            .with_context(|| format!("expected at least one || in {:?}", uuid))?;

        let id = id
            .parse::<i32>()
            .with_context(|| format!("ID (the last part) is not a i32 in {:?}", uuid))?;

        Ok((service.to_string(), id))
    }
}
