use crate::{
    Event,
    actor::{CanStop, TryWantsTrySatisfy},
    event_queue::EventQueue,
    sansio::{Satisfy, UnixSocketOneshotWriter, UnixSocketReader, Wants},
    user_data::ModuleId,
    utils::{StringRef, StringRefExt as _, getenv},
};
use anyhow::{Context, Result, bail};
use buffer::{Buffer, NiriEvent};
use rustix::net::SocketAddrUnix;

mod buffer;

enum State {
    Writer(Box<UnixSocketOneshotWriter>),
    Reader(Box<UnixSocketReader>),
    Stopped,
}

pub(crate) struct Niri {
    state: State,
    buffer: Buffer,
    layouts: Vec<String>,
}

impl Niri {
    pub(crate) fn new() -> Self {
        Self::try_new().unwrap_or_else(|err| {
            log::error!("{err:?}");
            Self::stopped()
        })
    }

    fn try_new() -> Result<Self> {
        let path = getenv(c"NIRI_SOCKET").context("no $NIRI_SOCKET")?;
        let addr = SocketAddrUnix::new(path)?;

        Ok(Self {
            state: State::Writer(Box::new(UnixSocketOneshotWriter::new(
                addr,
                "\"EventStream\"\n",
            )?)),
            buffer: Buffer::new(),
            layouts: vec![],
        })
    }

    const fn stopped() -> Self {
        Self {
            state: State::Stopped,
            buffer: Buffer::new(),
            layouts: vec![],
        }
    }

    fn process(&mut self, buf: &[u8], events: &mut EventQueue) -> Result<()> {
        let niri_events = self.buffer.push(buf)?;
        let mut layouts = None;
        let mut current_layout_idx = None;

        for event in niri_events {
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
            let mut lang = self
                .layouts
                .get(current_layout_idx)
                .context("no such layout idx")?
                .as_str();

            if lang == "English (US)" {
                lang = "EN";
            } else if lang == "Polish" {
                lang = "PL";
            } else {
                lang = "??";
            }

            events.push_back(Event::Language {
                lang: StringRef::new(lang),
            });
        }

        Ok(())
    }
}

impl TryWantsTrySatisfy for Niri {
    const ID: ModuleId = ModuleId::Niri;
    type Output = ();

    fn try_wants(&mut self) -> Result<Option<Wants>> {
        match &mut self.state {
            State::Writer(writer) => Ok(writer.wants()),
            State::Reader(reader) => Ok(reader.wants()),
            State::Stopped => Ok(None),
        }
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, events: &mut EventQueue) -> Result<Self::Output> {
        match &mut self.state {
            State::Writer(writer) => match satisfy {
                Satisfy::Socket(res) => {
                    let fd = res?;
                    writer.satisfy_socket(fd)?;
                }

                Satisfy::Connect(res) => {
                    res?;
                    writer.satisfy_connect()?;
                }

                Satisfy::Write(res) => {
                    let _ = res?;
                    writer.satisfy_write()?;
                    self.state = State::Reader(Box::new(UnixSocketReader::new_connected_from_fd(
                        writer.fd()?,
                    )));
                }

                _ => bail!("Niri writer only accepts Socket, Connect and Write, got: {satisfy:?}"),
            },

            State::Reader(reader) => match satisfy {
                Satisfy::Socket(res) => {
                    let fd = res?;
                    reader.satisfy_socket(fd)?;
                }

                Satisfy::Connect(res) => {
                    res?;
                    reader.satisfy_connect()?;
                }

                Satisfy::Read(res) => {
                    let bytes_read = res?;
                    let (buf, len) = reader.satisfy_read(bytes_read)?;
                    let buf = buf.get(..len).context("buf is too short")?;
                    self.process(buf, events)?;
                }

                _ => bail!("Niri reader only accepts Socket, Connect and Read, got: {satisfy:?}"),
            },

            State::Stopped => {}
        }

        Ok(())
    }
}

impl CanStop for Niri {
    fn stopped(&mut self) -> Self {
        Self::stopped()
    }
}
