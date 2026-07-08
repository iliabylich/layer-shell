use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use anyhow::{Context as _, Result};
use microjson::JSONValue;

use crate::utils::get_json;

pub(crate) struct Buffer {
    queue: Vec<u8>,
}

impl Buffer {
    pub(crate) const fn new() -> Self {
        Self { queue: vec![] }
    }

    pub(crate) fn push(&mut self, buf: &[u8]) -> Result<Vec<NiriEvent>> {
        let mut events = vec![];
        self.queue.extend(buf.iter());

        let mut q = self.queue.as_slice();

        while let Some((event, rem)) = NiriEvent::cut(q) {
            q = rem;

            if let Some(event) = event? {
                events.push(event);
            }
        }

        self.queue = q.to_vec();

        Ok(events)
    }
}

#[derive(Debug)]
pub(crate) enum NiriEvent {
    KeyboardLayoutsChanged { keyboard_layouts: KeyboardLayouts },
    KeyboardLayoutSwitched { idx: usize },
}

impl NiriEvent {
    fn from_json(bytes: &[u8]) -> Result<Option<Self>> {
        let s = core::str::from_utf8(bytes)?;
        let json = JSONValue::load(s);

        match json.get_key_value("KeyboardLayoutsChanged") {
            Ok(json) => {
                let event = Self::parse_keyboard_layouts_changed(&json)?;
                return Ok(Some(event));
            }
            Err(err) => match err {
                microjson::JSONParsingError::KeyNotFound => {}
                err => return Err(anyhow::anyhow!(err)),
            },
        }

        match json.get_key_value("KeyboardLayoutSwitched") {
            Ok(json) => {
                let event = Self::parse_keyword_layout_switched(&json)?;
                return Ok(Some(event));
            }
            Err(err) => match err {
                microjson::JSONParsingError::KeyNotFound => {}
                err => return Err(anyhow::anyhow!(err)),
            },
        }

        Ok(None)
    }

    fn parse_keyboard_layouts_changed(json: &JSONValue) -> Result<Self> {
        let keyboard_layouts = json
            .get_key_value("keyboard_layouts")
            .map_err(|err| anyhow::anyhow!(err))?;
        let names = keyboard_layouts
            .get_key_value("names")
            .map_err(|err| anyhow::anyhow!(err))?
            .iter_array()
            .map_err(|err| anyhow::anyhow!(err))?
            .map(|name| {
                name.read_string()
                    .map(ToString::to_string)
                    .map_err(|err| anyhow::anyhow!(err))
            })
            .collect::<Result<Vec<_>>>()?;

        let current_idx = get_json!(keyboard_layouts, "current_idx", read_integer);
        let current_idx = usize::try_from(current_idx).context("negative keyboard current_idx")?;
        Ok(Self::KeyboardLayoutsChanged {
            keyboard_layouts: KeyboardLayouts { names, current_idx },
        })
    }

    fn parse_keyword_layout_switched(json: &JSONValue) -> Result<Self> {
        let idx = get_json!(json, "idx", read_integer);
        let idx = usize::try_from(idx).context("negative keyboard idx")?;
        Ok(Self::KeyboardLayoutSwitched { idx })
    }

    fn cut(bytes: &[u8]) -> Option<(Result<Option<Self>>, &[u8])> {
        let nl_idx = bytes.iter().position(|b| *b == b'\n')?;

        let (pre, post) = bytes.split_at(nl_idx);
        let post = unsafe { post.get_unchecked(1..) };

        Some((Self::from_json(pre), post))
    }
}

#[derive(Debug)]
pub(crate) struct KeyboardLayouts {
    pub(crate) names: Vec<String>,
    pub(crate) current_idx: usize,
}
