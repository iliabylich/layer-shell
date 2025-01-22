use anyhow::{Context as _, Result};

pub(crate) struct UUID;

impl UUID {
    pub(crate) fn encode(service: impl AsRef<str>, path: impl AsRef<str>, id: i32) -> String {
        format!("{}||{}||{}", service.as_ref(), path.as_ref(), id)
    }

    pub(crate) fn decode(uuid: impl AsRef<str>) -> Result<(String, String, i32)> {
        let uuid = uuid.as_ref();

        let (service, rest) = uuid
            .split_once("||")
            .with_context(|| format!("expected at least one || in {:?}", uuid))?;
        let (path, id) = rest
            .split_once("||")
            .with_context(|| format!("expected at least two || separators in {:?}", uuid))?;

        let id = id
            .parse::<i32>()
            .with_context(|| format!("ID (the last part) is not a i32 in {:?}", uuid))?;

        Ok((service.to_string(), path.to_string(), id))
    }
}
