use crate::{
    Event,
    event_queue::EventQueue,
    modules::FallibleModule,
    sansio::{Satisfy, UnixSocketOneshotWriter, UnixSocketReader, Wants},
    unix_socket::new_unix_socket,
    utils::StringRef,
};
use anyhow::{Context, Result};
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
    pub(crate) fn new() -> Result<Self> {
        let Ok(path) = std::env::var("NIRI_SOCKET") else {
            log::error!("no $NIRI_SOCKET");
            return Ok(Self {
                state: None,
                queue: vec![],
                layouts: vec![],
            });
        };

        let addr = new_unix_socket(path.as_bytes())?;
        Ok(Self {
            state: Some(State::Writer(Box::new(UnixSocketOneshotWriter::new(
                addr,
                "\"EventStream\"\n",
            )?))),
            queue: vec![],
            layouts: vec![],
        })
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<()> {
        let Some(state) = &mut self.state else {
            return Ok(());
        };

        match state {
            State::Writer(writer) => {
                writer.satisfy(satisfy, res)?;

                if matches!(satisfy, Satisfy::Write) {
                    self.state = Some(State::Reader(Box::new(
                        UnixSocketReader::new_connected_from_fd(writer.fd()),
                    )));
                }

                Ok(())
            }
            State::Reader(reader) => {
                let Some((buf, len)) = reader.satisfy(satisfy, res) else {
                    return Ok(());
                };

                self.parse_event(buf.get(..len).context("buf is too short")?)?;
                Ok(())
            }
        }
    }

    fn parse_event(&mut self, buf: &[u8]) -> Result<()> {
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

            let post = post.get(1..).context("malformed event")?;
            q = post;
        }

        self.queue = q.to_vec();

        if let Some(layouts) = layouts {
            self.layouts = layouts;
        }
        if let Some(current_layout_idx) = current_layout_idx {
            let lang = self
                .layouts
                .get(current_layout_idx)
                .context("no such layout idx")?;
            EventQueue::push_back(Event::Language {
                lang: StringRef::new(lang)?,
            });
        }

        Ok(())
    }
}

impl FallibleModule for Niri {
    const NAME: &str = "Niri";
    type Output = ();

    fn try_wants(&mut self) -> Result<Option<Wants>> {
        let Some(state) = self.state.as_mut() else {
            return Ok(None);
        };

        match state {
            State::Writer(writer) => writer.wants(),
            State::Reader(reader) => Ok(reader.wants()),
        }
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<Self::Output>> {
        self.try_satisfy(satisfy, res)?;
        Ok(None)
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
