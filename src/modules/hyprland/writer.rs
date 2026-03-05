use crate::{
    Event,
    modules::hyprland::{
        oneshot_writer::OneshotWriter,
        resources::{
            ActiveWorkspaceResource, CapsLockResource, DevicesResource, DispatchResource,
            WorkspacesResource, WriterResource,
        },
        state::HyprlandState,
    },
    unix_socket::new_unix_socket,
};
use anyhow::Result;
use libc::sockaddr_un;
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

pub(crate) struct HyprlandWriter {
    current: Option<OneshotWriter>,
    queue: VecDeque<Box<dyn WriterResource>>,
    addr: sockaddr_un,
    state: Rc<RefCell<HyprlandState>>,
}

impl HyprlandWriter {
    pub(crate) fn new(
        xdg_runtime_dir: &str,
        hyprland_instance_signature: &str,
        state: Rc<RefCell<HyprlandState>>,
    ) -> Self {
        let addr = new_unix_socket(
            format!("{xdg_runtime_dir}/hypr/{hyprland_instance_signature}/.socket.sock").as_bytes(),
        );

        Self {
            current: None,
            queue: VecDeque::new(),
            addr,
            state,
        }
    }

    pub(crate) fn init(&mut self) {
        self.enqueue(Box::new(WorkspacesResource));
        self.enqueue(Box::new(ActiveWorkspaceResource));
        self.enqueue(Box::new(DevicesResource));
    }

    fn enqueue(&mut self, resource: Box<dyn WriterResource>) {
        if self.current.is_none() {
            let mut current = OneshotWriter::new(self.addr, resource);
            current.init();
            self.current = Some(current);
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

    fn try_process(&mut self, op: u8, res: i32, events: &mut Vec<Event>) -> Result<()> {
        if let Some(current) = self.current.as_mut()
            && let Some(diff) = current.process(op, res)?
        {
            let mut state = self.state.borrow_mut();
            if let Some(event) = state.apply(diff) {
                events.push(event);
            }
            self.current = None;
        }

        if self.current.is_none()
            && let Some(resource) = self.queue.pop_front()
        {
            let mut next = OneshotWriter::new(self.addr, resource);
            next.init();
            self.current = Some(next);
        }

        Ok(())
    }

    pub(crate) fn process(&mut self, op: u8, res: i32, events: &mut Vec<Event>) {
        if let Err(err) = self.try_process(op, res, events) {
            log::error!("{err:?}")
        }
    }
}
