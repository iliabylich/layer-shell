use anyhow::{Context as _, Result};

#[expect(clippy::upper_case_acronyms)]
pub(crate) struct UUID;

impl UUID {
    pub(crate) fn encode(service: &str, id: i32) -> String {
        format!("{}**{}", service, id)
    }

    pub(crate) fn decode(uuid: &str) -> Result<(&str, i32)> {
        let (service, rest) = uuid
            .split_once("**")
            .with_context(|| format!("expected at least one || in {:?}", uuid))?;

        let id = rest
            .parse::<i32>()
            .with_context(|| format!("ID (the last part) is not a i32 in {:?}", uuid))?;

        Ok((service, id))
    }
}
