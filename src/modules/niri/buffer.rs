use anyhow::Result;
use serde::Deserialize;
use serde_json::Value;

pub(crate) struct Buffer {
    queue: Vec<u8>,
}

impl Buffer {
    pub(crate) fn new() -> Self {
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

#[derive(Deserialize, Debug)]
pub(crate) enum NiriEvent {
    KeyboardLayoutsChanged { keyboard_layouts: KeyboardLayouts },
    KeyboardLayoutSwitched { idx: usize },
}

impl NiriEvent {
    fn parse(bytes: &[u8]) -> Result<Option<Self>> {
        let value: Value = serde_json::from_slice(bytes)?;
        let Ok(event) = serde_json::from_value::<Self>(value) else {
            return Ok(None);
        };
        Ok(Some(event))
    }

    fn cut(bytes: &[u8]) -> Option<(Result<Option<Self>>, &[u8])> {
        let nl_idx = bytes.iter().position(|b| *b == b'\n')?;

        let (pre, post) = bytes.split_at(nl_idx);
        let post = unsafe { post.get_unchecked(1..) };

        Some((NiriEvent::parse(pre), post))
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct KeyboardLayouts {
    pub(crate) names: Vec<String>,
    pub(crate) current_idx: usize,
}
