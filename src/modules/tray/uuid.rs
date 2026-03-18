use anyhow::{Context as _, Result};

use crate::ffi::ShortString;

#[expect(clippy::upper_case_acronyms)]
pub(crate) struct UUID;

impl UUID {
    pub(crate) fn encode(service: ShortString, id: i32) -> ShortString {
        let uuid = format!("{}**{}", service.as_str(), id);
        ShortString::from(uuid.as_str())
    }

    pub(crate) fn decode(uuid: ShortString) -> Result<(ShortString, i32)> {
        let uuid = uuid.as_str();

        let (service, rest) = uuid
            .split_once("**")
            .with_context(|| format!("expected at least one || in {:?}", uuid))?;

        let id = rest
            .parse::<i32>()
            .with_context(|| format!("ID (the last part) is not a i32 in {:?}", uuid))?;

        Ok((ShortString::from(service), id))
    }
}
