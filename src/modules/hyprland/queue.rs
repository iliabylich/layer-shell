use crate::modules::hyprland::resources::{
    ActiveWorkspaceResource, CapsLockResource, DevicesResource, DispatchResource,
    WorkspacesResource, WriterResource,
};
use core::cell::RefCell;
use std::{collections::VecDeque, rc::Rc};

pub(crate) struct HyprlandQueue {
    q: Rc<RefCell<VecDeque<Box<dyn WriterResource>>>>,
    dummy: bool,
}

impl HyprlandQueue {
    pub(crate) fn dummy() -> Self {
        let mut this = Self::new();
        this.dummy = true;
        this
    }

    pub(crate) fn new() -> Self {
        let mut q: VecDeque<Box<dyn WriterResource>> = VecDeque::new();
        q.push_back(Box::new(ActiveWorkspaceResource));
        q.push_back(Box::new(DevicesResource));
        q.push_back(Box::new(WorkspacesResource));

        Self {
            q: Rc::new(RefCell::new(q)),
            dummy: false,
        }
    }

    pub(crate) fn push_back(&self, resource: Box<dyn WriterResource>) {
        if self.dummy {
            return;
        }

        let mut q = self.q.borrow_mut();
        q.push_back(resource);
    }

    pub(crate) fn enqueue_get_caps_lock(&self) {
        self.push_back(Box::new(CapsLockResource));
    }

    pub(crate) fn enqueue_dispatch(&self, cmd: String) {
        self.push_back(Box::new(DispatchResource::new(cmd)));
    }

    pub(crate) fn pop_front(&self) -> Option<Box<dyn WriterResource>> {
        let mut q = self.q.borrow_mut();
        q.pop_front()
    }

    pub(crate) fn copy(&self) -> Self {
        Self {
            q: Rc::clone(&self.q),
            dummy: self.dummy,
        }
    }
}
