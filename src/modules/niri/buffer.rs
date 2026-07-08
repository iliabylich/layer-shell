use alloc::{
    string::{String, ToString as _},
    vec,
    vec::Vec,
};
use anyhow::{Context as _, Result};
use jzon::JsonValue;

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
        let json: JsonValue = jzon::parse(s)?;

        let event = if json.has_key("KeyboardLayoutsChanged") {
            Self::parse_keyboard_layouts_changed(&json)
                .context("malformed KeyboardLayoutsChanged event")?
        } else if json.has_key("KeyboardLayoutSwitched") {
            Self::parse_keyword_layout_switched(&json)
                .context("malformed KeyboardLayoutSwitched event")?
        } else {
            return Ok(None);
        };

        Ok(Some(event))
    }

    fn parse_keyboard_layouts_changed(json: &JsonValue) -> Result<Self> {
        let keyboard_layouts_changed = get_json!(json, "KeyboardLayoutsChanged", as_object);
        let keyboard_layouts = get_json!(keyboard_layouts_changed, "keyboard_layouts", as_object);
        let names = get_json!(keyboard_layouts, "names", as_array)
            .iter()
            .map(|name| {
                let name = name.as_str().context("names contains non-string")?;
                Ok(name.to_string())
            })
            .collect::<Result<Vec<_>>>()?;
        let current_idx = get_json!(keyboard_layouts, "current_idx", as_usize);
        Ok(Self::KeyboardLayoutsChanged {
            keyboard_layouts: KeyboardLayouts { names, current_idx },
        })
    }

    fn parse_keyword_layout_switched(json: &JsonValue) -> Result<Self> {
        let keyboard_layout_switched = get_json!(json, "KeyboardLayoutSwitched", as_object);
        let idx = get_json!(keyboard_layout_switched, "idx", as_usize);
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
