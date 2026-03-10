use crate::{
    event_queue::EventQueue,
    modules::{
        Module,
        hyprland::{HyprlandQueue, resources::WriterResource, state::HyprlandState},
    },
    sansio::{Satisfy, UnixSocketOneshotWriter, Wants},
    user_data::ModuleId,
};
use anyhow::{Context as _, Result};
use libc::sockaddr_un;

pub(crate) struct HyprlandWriter {
    current: Option<(UnixSocketOneshotWriter, Box<dyn WriterResource>)>,
    queue: HyprlandQueue,
    addr: sockaddr_un,
    state: HyprlandState,
    events: EventQueue,
}

impl HyprlandWriter {
    pub(crate) fn new(
        addr: sockaddr_un,
        state: HyprlandState,
        events: EventQueue,
        queue: HyprlandQueue,
    ) -> Self {
        let mut this = Self {
            current: None,
            queue,
            addr,
            state,
            events,
        };
        this.pop_from_queue_into_current();
        this
    }

    fn pop_from_queue_into_current(&mut self) {
        if self.current.is_none()
            && let Some(resource) = self.queue.pop_front()
        {
            self.current = Some((
                UnixSocketOneshotWriter::new(self.addr, resource.command().as_ref()),
                resource,
            ));
        }
    }
}

impl Module for HyprlandWriter {
    type Output = ();
    type Error = anyhow::Error;

    const MODULE_ID: ModuleId = ModuleId::HyprlandWriter;

    fn wants(&mut self) -> Wants {
        self.pop_from_queue_into_current();

        let Some((socket_writer, _)) = &mut self.current else {
            return Wants::Nothing;
        };

        socket_writer.wants()
    }

    fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Self::Output, Self::Error> {
        let Some((socket_writer, resource)) = &mut self.current else {
            return Ok(());
        };

        let Some(buf) = socket_writer.satisfy(satisfy, res)? else {
            return Ok(());
        };

        let json = std::str::from_utf8(buf).context("decoding error")?;
        let diff = resource.parse(json).context("parse error")?;

        self.current = None;
        self.pop_from_queue_into_current();

        let Some(diff) = diff else {
            return Ok(());
        };

        if let Some(event) = self.state.apply(diff) {
            self.events.push_back(event);
        }

        Ok(())
    }

    fn tick(&mut self, _tick: u64) {}
}
