use crate::{
    external::exit,
    utils::{ArrayWriter, StringRef, StringRefExt as _},
};
use anyhow::{Context as _, Result};
use core::fmt::Write;

#[expect(clippy::upper_case_acronyms)]
pub(crate) struct UUID;

impl UUID {
    pub(crate) fn encode(service: &str, id: i32) -> StringRef {
        let mut buf = [0; 1_024];
        let mut w = ArrayWriter::new(&mut buf);
        write!(&mut w, "{service}**{id}").unwrap_or_else(|_| {
            log::error!("UUID doesn't fit into StringRef");
            unsafe { exit(1) };
        });
        let uuid = w.as_str().unwrap_or_else(|_| unreachable!());
        StringRef::new(uuid)
    }

    pub(crate) fn decode(uuid: &str) -> Result<(StringRef, i32)> {
        let (service, rest) = uuid.split_once("**").context("expected at least one ||")?;

        let id = rest
            .parse::<i32>()
            .context("ID (the last part) is not a i32")?;

        Ok((StringRef::new(service), id))
    }
}
