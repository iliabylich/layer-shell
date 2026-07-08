use crate::utils::{StringRef, StringRefExt as _};
use anyhow::{Context as _, Result};

#[expect(clippy::upper_case_acronyms)]
pub(crate) struct UUID;

impl UUID {
    pub(crate) fn encode(service: &str, id: i32) -> StringRef {
        let uuid = format!("{service}**{id}");
        StringRef::new(uuid.as_str())
    }

    pub(crate) fn decode(uuid: &str) -> Result<(StringRef, i32)> {
        let (service, rest) = uuid.split_once("**").context("expected at least one ||")?;

        let id = rest
            .parse::<i32>()
            .context("ID (the last part) is not a i32")?;

        Ok((StringRef::new(service), id))
    }
}
