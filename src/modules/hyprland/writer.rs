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
    dead: bool,
}

impl HyprlandWriter {
    pub(crate) fn new(addr: sockaddr_un, state: HyprlandState, queue: HyprlandQueue) -> Self {
        let mut this = Self {
            current: None,
            queue,
            addr,
            state,
            dead: false,
        };
        this.pop_from_queue_into_current();
        this
    }

    pub(crate) fn dummy(state: HyprlandState, queue: HyprlandQueue) -> Self {
        Self {
            current: None,
            queue,
            addr: unsafe { core::mem::MaybeUninit::zeroed().assume_init() },
            state,
            dead: true,
        }
    }

    fn pop_from_queue_into_current(&mut self) {
        if self.current.is_none()
            && let Some(resource) = self.queue.pop_front()
        {
            self.current = Some((
                UnixSocketOneshotWriter::new(self.addr, resource.command()),
                resource,
            ));
        }
    }

    pub(crate) const fn module_id(&self) -> ModuleId {
        ModuleId::HyprlandWriter
    }

    pub(crate) fn wants(&mut self) -> Wants {
        if self.dead {
            return Wants::Nothing;
        }

        self.pop_from_queue_into_current();

        let Some((socket_writer, _)) = &mut self.current else {
            return Wants::Nothing;
        };

        socket_writer.wants()
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<()> {
        let Some((socket_writer, resource)) = &mut self.current else {
            return Ok(());
        };

        let Some(buf) = socket_writer.satisfy(satisfy, res)? else {
            return Ok(());
        };

        let json = core::str::from_utf8(buf).context("decoding error")?;
        let diff = resource.parse(json).context("parse error")?;

        self.current = None;
        self.pop_from_queue_into_current();

        let Some(diff) = diff else {
            return Ok(());
        };

        if let Some(event) = self.state.apply(diff) {
            EventQueue::push_back(event);
        }

        Ok(())
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) {
        if self.dead {
            return;
        }

        if let Err(err) = self.try_satisfy(satisfy, res) {
            log::error!("HyprlandReader has crashed: {satisfy:?} {res} {err:?}");
            self.current = None;
            self.dead = true;
        }
    }
}
