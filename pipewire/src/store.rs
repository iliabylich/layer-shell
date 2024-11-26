use pipewire::{metadata::Metadata, node::Node, proxy::Listener};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub(crate) struct Store {
    nodes: Rc<RefCell<HashMap<u32, Rc<Node>>>>,
    meta: Rc<RefCell<HashMap<u32, Metadata>>>,
    listeners: Rc<RefCell<HashMap<u32, Vec<Box<dyn Listener>>>>>,
    sink_name_to_id: Rc<RefCell<HashMap<String, u32>>>,
    default_sink_name: Rc<RefCell<Option<String>>>,
}

impl Store {
    pub(crate) fn new() -> Self {
        Self {
            nodes: Rc::new(RefCell::new(HashMap::new())),
            meta: Rc::new(RefCell::new(HashMap::new())),
            listeners: Rc::new(RefCell::new(HashMap::new())),
            sink_name_to_id: Rc::new(RefCell::new(HashMap::new())),
            default_sink_name: Rc::new(RefCell::new(None)),
        }
    }

    pub(crate) fn add_node(&self, id: u32, node: Node) {
        self.nodes.borrow_mut().insert(id, Rc::new(node));
    }

    pub(crate) fn add_meta(&self, id: u32, meta: Metadata) {
        self.meta.borrow_mut().insert(id, meta);
    }

    pub(crate) fn add_listener(&self, id: u32, listener: Box<dyn Listener>) {
        self.listeners
            .borrow_mut()
            .entry(id)
            .or_default()
            .push(listener);
    }

    pub(crate) fn set_default_sink_name(&self, name: String) {
        *self.default_sink_name.borrow_mut() = Some(name);
    }

    pub(crate) fn add_sink_name(&self, name: impl AsRef<str>, id: u32) {
        self.sink_name_to_id
            .borrow_mut()
            .insert(name.as_ref().to_string(), id);
    }

    pub(crate) fn default_sink(&self) -> Option<Rc<Node>> {
        let default_sink_name = self.default_sink_name.borrow();
        let name = default_sink_name.as_ref()?;

        let sink_name_to_id = self.sink_name_to_id.borrow();
        let id = sink_name_to_id.get(name)?;

        let nodes = self.nodes.borrow();
        let node = nodes.get(id)?;
        Some(Rc::clone(node))
    }

    pub(crate) fn shallow_clone(&self) -> Self {
        Self {
            nodes: Rc::clone(&self.nodes),
            meta: Rc::clone(&self.meta),
            listeners: Rc::clone(&self.listeners),
            sink_name_to_id: Rc::clone(&self.sink_name_to_id),
            default_sink_name: Rc::clone(&self.default_sink_name),
        }
    }
}
