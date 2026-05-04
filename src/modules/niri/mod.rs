use crate::{
    Event,
    event_queue::EventQueue,
    modules::FallibleModule,
    sansio::{Satisfy, UnixSocketOneshotWriter, UnixSocketReader, Wants},
    unix_socket::new_unix_socket,
    utils::StringRef,
};
use anyhow::{Context, Result};
use buffer::{Buffer, NiriEvent};

mod buffer;

enum State {
    Writer(Box<UnixSocketOneshotWriter>),
    Reader(Box<UnixSocketReader>),
}

pub(crate) struct Niri {
    state: State,
    buffer: Buffer,
    layouts: Vec<String>,
}

impl Niri {
    pub(crate) fn new() -> Result<Self> {
        let path = std::env::var("NIRI_SOCKET").context("no $NIRI_SOCKET")?;

        let addr = new_unix_socket(path.as_bytes())?;
        Ok(Self {
            state: State::Writer(Box::new(UnixSocketOneshotWriter::new(
                addr,
                "\"EventStream\"\n",
            )?)),
            buffer: Buffer::new(),
            layouts: vec![],
        })
    }

    fn process(&mut self, buf: &[u8]) -> Result<()> {
        let events = self.buffer.push(buf)?;
        let mut layouts = None;
        let mut current_layout_idx = None;

        for event in events {
            match event {
                NiriEvent::KeyboardLayoutsChanged { keyboard_layouts } => {
                    layouts = Some(keyboard_layouts.names);
                    current_layout_idx = Some(keyboard_layouts.current_idx);
                }
                NiriEvent::KeyboardLayoutSwitched { idx } => {
                    current_layout_idx = Some(idx);
                }
            }
        }

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
        match &mut self.state {
            State::Writer(writer) => writer.wants(),
            State::Reader(reader) => Ok(reader.wants()),
        }
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<Self::Output>> {
        match &mut self.state {
            State::Writer(writer) => {
                writer.satisfy(satisfy, res)?;

                if matches!(satisfy, Satisfy::Write) {
                    self.state = State::Reader(Box::new(UnixSocketReader::new_connected_from_fd(
                        writer.fd(),
                    )));
                }
            }
            State::Reader(reader) => {
                let Some((buf, len)) = reader.try_satisfy(satisfy, res)? else {
                    return Ok(None);
                };
                let buf = buf.get(..len).context("buf is too short")?;
                self.process(buf)?;
            }
        }

        Ok(None)
    }
}
