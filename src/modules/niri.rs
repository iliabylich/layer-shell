use crate::{
    Event,
    event_queue::EventQueue,
    sansio::{Satisfy, UnixSocketOneshotWriter, UnixSocketReader, Wants},
    unix_socket::new_unix_socket,
    user_data::ModuleId,
    utils::StringRef,
};
use anyhow::Result;
use serde::Deserialize;
use serde_json::Value;

enum State {
    Writer(Box<UnixSocketOneshotWriter>),
    Reader(Box<UnixSocketReader>),
}

pub(crate) struct Niri {
    state: Option<State>,
    queue: Vec<u8>,
    layouts: Vec<String>,
}

impl Niri {
    pub(crate) fn new() -> Self {
        let Ok(path) = std::env::var("NIRI_SOCKET") else {
            log::error!("no $NIRI_SOCKET");
            return Self {
                state: None,
                queue: vec![],
                layouts: vec![],
            };
        };

        let addr = new_unix_socket(path.as_bytes());
        Self {
            state: Some(State::Writer(Box::new(UnixSocketOneshotWriter::new(
                addr,
                StringRef::new("\"EventStream\"\n"),
            )))),
            queue: vec![],
            layouts: vec![],
        }
    }

    pub(crate) fn module_id(&self) -> ModuleId {
        ModuleId::Niri
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        match self.state.as_mut()? {
            State::Writer(writer) => writer.wants(),
            State::Reader(reader) => reader.wants(),
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) {
        let Some(state) = &mut self.state else {
            return;
        };

        match state {
            State::Writer(writer) => {
                if let Err(err) = writer.satisfy(satisfy, res) {
                    log::error!(target: "Niri", "{err:?}");
                    self.state = None;
                    return;
                }

                if matches!(satisfy, Satisfy::Write) {
                    self.state = Some(State::Reader(Box::new(
                        UnixSocketReader::new_connected_from_fd(writer.fd()),
                    )));
                }
            }
            State::Reader(reader) => {
                let Some((buf, len)) = reader.satisfy(satisfy, res) else {
                    return;
                };

                if let Err(err) = self.on_data(&buf[..len]) {
                    log::error!(target: "Niri", "{err:?}");
                    self.state = None;
                }
            }
        }
    }

    fn on_data(&mut self, buf: &[u8]) -> Result<()> {
        self.queue.extend(buf.iter());

        let mut q = self.queue.as_slice();
        let mut layouts = None;
        let mut current_layout_idx = None;

        while let Some(nl_idx) = q.iter().position(|b| *b == b'\n') {
            let (pre, post) = q.split_at(nl_idx);
            match parse_line(pre)? {
                Some(NiriEvent::KeyboardLayoutsChanged { keyboard_layouts }) => {
                    layouts = Some(keyboard_layouts.names);
                    current_layout_idx = Some(keyboard_layouts.current_idx);
                }
                Some(NiriEvent::KeyboardLayoutSwitched { idx }) => {
                    current_layout_idx = Some(idx);
                }
                None => {}
            }

            let post = &post[1..];
            q = post;
        }

        self.queue = q.to_vec();

        if let Some(layouts) = layouts {
            self.layouts = layouts;
        }
        if let Some(current_layout_idx) = current_layout_idx {
            let lang = &self.layouts[current_layout_idx];
            EventQueue::push_back(Event::Language {
                lang: StringRef::new(lang),
            });
        }

        Ok(())
    }
}

#[derive(Deserialize, Debug)]
enum NiriEvent {
    KeyboardLayoutsChanged { keyboard_layouts: KeyboardLayouts },
    KeyboardLayoutSwitched { idx: usize },
}

#[derive(Deserialize, Debug)]
struct KeyboardLayouts {
    names: Vec<String>,
    current_idx: usize,
}

fn parse_line(line: &[u8]) -> Result<Option<NiriEvent>> {
    let value: Value = serde_json::from_slice(line)?;
    let Ok(event) = serde_json::from_value::<NiriEvent>(value) else {
        return Ok(None);
    };
    Ok(Some(event))
}
