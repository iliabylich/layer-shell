use crate::{
    event_queue::EventQueue,
    modules::hyprland::{HyprlandQueue, resources::WriterResource, state::HyprlandState},
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

    pub(crate) const fn module_id(&self) -> ModuleId {
        ModuleId::HyprlandWriter
    }

    pub(crate) fn wants(&mut self) -> Wants {
        self.pop_from_queue_into_current();

        let Some((socket_writer, _)) = &mut self.current else {
            return Wants::Nothing;
        };

        socket_writer.wants()
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<()> {
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
}
