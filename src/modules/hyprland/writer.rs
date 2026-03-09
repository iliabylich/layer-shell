use crate::{
    Event,
    modules::{
        Module,
        hyprland::{
            resources::{
                ActiveWorkspaceResource, CapsLockResource, DevicesResource, DispatchResource,
                WorkspacesResource, WriterResource,
            },
            state::HyprlandState,
        },
    },
    sansio::{Satisfy, UnixSocketOneshotWriter, Wants},
    user_data::ModuleId,
};
use anyhow::{Context as _, Result};
use libc::sockaddr_un;
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

pub(crate) struct HyprlandWriter {
    current: Option<(UnixSocketOneshotWriter, Box<dyn WriterResource>)>,
    queue: VecDeque<Box<dyn WriterResource>>,
    addr: sockaddr_un,
    state: Rc<RefCell<HyprlandState>>,
}

impl HyprlandWriter {
    fn enqueue(&mut self, resource: Box<dyn WriterResource>) {
        if self.current.is_none() {
            self.current = Some((
                UnixSocketOneshotWriter::new(self.addr, resource.command().as_ref()),
                resource,
            ));
        } else {
            self.queue.push_back(resource);
        }
    }

    pub(crate) fn enqueue_get_caps_lock(&mut self) {
        self.enqueue(Box::new(CapsLockResource));
    }

    pub(crate) fn enqueue_dispatch(&mut self, cmd: String) {
        self.enqueue(Box::new(DispatchResource::new(cmd)));
    }
}

impl Module for HyprlandWriter {
    type Input = (sockaddr_un, Rc<RefCell<HyprlandState>>);
    type Output = ();
    type Error = anyhow::Error;

    const MODULE_ID: ModuleId = ModuleId::HyprlandWriter;

    fn new((addr, state): Self::Input) -> Self {
        let mut queue: VecDeque<Box<dyn WriterResource>> = VecDeque::new();

        queue.push_back(Box::new(ActiveWorkspaceResource));
        queue.push_back(Box::new(DevicesResource));

        let resource = Box::new(WorkspacesResource);

        Self {
            current: Some((
                UnixSocketOneshotWriter::new(addr, resource.command().as_ref()),
                resource,
            )),
            queue,
            addr,
            state,
        }
    }

    fn wants(&mut self) -> Wants {
        let Some((socket_writer, _)) = &mut self.current else {
            return Wants::Nothing;
        };

        socket_writer.wants()
    }

    fn satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
        events: &mut Vec<Event>,
    ) -> Result<Self::Output, Self::Error> {
        let Some((socket_writer, resource)) = &mut self.current else {
            return Ok(());
        };

        let Some(buf) = socket_writer.satisfy(satisfy, res)? else {
            return Ok(());
        };

        let json = std::str::from_utf8(buf).context("decoding error")?;
        let diff = resource.parse(json).context("parse error")?;

        self.current = None;
        if let Some(next) = self.queue.pop_front() {
            self.current = Some((
                UnixSocketOneshotWriter::new(self.addr, next.command().as_ref()),
                next,
            ));
        }

        let Some(diff) = diff else {
            return Ok(());
        };

        let mut state = self.state.borrow_mut();
        if let Some(event) = state.apply(diff) {
            events.push(event);
        }

        Ok(())
    }

    fn tick(&mut self, _tick: u64) {}
}
