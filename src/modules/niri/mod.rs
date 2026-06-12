use crate::{
    Event,
    event_queue::EventQueue,
    modules::FallibleModule,
    sansio::{Satisfy, UnixSocketOneshotWriter, UnixSocketReader, Wants},
    unix_socket::new_unix_socket,
    user_data::ModuleId,
    utils::{StringRef, StringRefExt as _},
};
use anyhow::{Context, Result, bail};
use buffer::{Buffer, NiriEvent};

mod buffer;

enum State {
    Writer(Box<UnixSocketOneshotWriter>),
    Reader(Box<UnixSocketReader>),
    Dummy,
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
            Self::dummy()
        })
    }

    fn try_new() -> Result<Self> {
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

    const fn dummy() -> Self {
        Self {
            state: State::Dummy,
            buffer: Buffer::new(),
            layouts: vec![],
        }
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
                lang: StringRef::new(lang),
            });
        }

        Ok(())
    }
}

impl FallibleModule for Niri {
    const MODULE_ID: ModuleId = ModuleId::Niri;
    type Output = ();

    fn wants(&mut self) -> Result<Option<Wants>> {
        match &mut self.state {
            State::Writer(writer) => Ok(Some(writer.wants()?)),
            State::Reader(reader) => Ok(Some(reader.wants())),
            State::Dummy => Ok(None),
        }
    }

    fn try_satisfy(&mut self, satisfy: Satisfy) -> Result<Option<Self::Output>> {
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
                        writer.fd(),
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
                    self.process(buf)?;
                }

                _ => bail!("Niri reader only accepts Socket, Connect and Read, got: {satisfy:?}"),
            },

            State::Dummy => {}
        }

        Ok(None)
    }
}
