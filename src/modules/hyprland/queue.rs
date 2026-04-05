use crate::modules::hyprland::resources::{
    ActiveWorkspaceResource, CapsLockResource, DevicesResource, DispatchResource,
    WorkspacesResource, WriterResource,
};
use std::collections::VecDeque;

pub(crate) struct HyprlandQueue {
    q: VecDeque<Box<dyn WriterResource>>,
    dummy: bool,
}
static mut QUEUE: HyprlandQueue = HyprlandQueue::empty();

impl HyprlandQueue {
    pub(crate) fn make_dummy() {
        unsafe { QUEUE.dummy = true }
    }

    const fn empty() -> Self {
        Self {
            q: VecDeque::new(),
            dummy: false,
        }
    }

    pub(crate) fn init() {
        let q = &mut unsafe { &mut QUEUE }.q;
        q.push_back(Box::new(ActiveWorkspaceResource));
        q.push_back(Box::new(DevicesResource));
        q.push_back(Box::new(WorkspacesResource));
    }

    pub(crate) fn push_back(resource: Box<dyn WriterResource>) {
        let q = unsafe { &mut QUEUE };

        if q.dummy {
            return;
        }

        q.q.push_back(resource);
    }

    pub(crate) fn enqueue_get_caps_lock() {
        Self::push_back(Box::new(CapsLockResource));
    }

    pub(crate) fn enqueue_dispatch(cmd: String) {
        Self::push_back(Box::new(DispatchResource::new(cmd)));
    }

    pub(crate) fn pop_front() -> Option<Box<dyn WriterResource>> {
        unsafe { QUEUE.q.pop_front() }
    }
}
