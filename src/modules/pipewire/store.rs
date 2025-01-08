use pipewire::{metadata::Metadata, node::Node, proxy::Listener};
use std::collections::HashMap;

pub(crate) struct Store {
    nodes: HashMap<u32, Node>,
    meta: HashMap<u32, Metadata>,
    listeners: HashMap<u32, Vec<Box<dyn Listener>>>,
    sink_name_to_id: HashMap<String, u32>,
    default_sink_name: Option<String>,
}

impl Store {
    pub(crate) fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            meta: HashMap::new(),
            listeners: HashMap::new(),
            sink_name_to_id: HashMap::new(),
            default_sink_name: None,
        }
    }

    pub(crate) fn add_node(&mut self, id: u32, node: Node) {
        self.nodes.insert(id, node);
    }

    pub(crate) fn add_meta(&mut self, id: u32, meta: Metadata) {
        self.meta.insert(id, meta);
    }

    pub(crate) fn add_listener(&mut self, id: u32, listener: Box<dyn Listener>) {
        self.listeners.entry(id).or_default().push(listener);
    }

    pub(crate) fn set_default_sink_name(&mut self, name: String) {
        self.default_sink_name = Some(name);
    }

    pub(crate) fn add_sink_name(&mut self, name: impl AsRef<str>, id: u32) {
        self.sink_name_to_id.insert(name.as_ref().to_string(), id);
    }

    pub(crate) fn default_sink(&self) -> Option<&Node> {
        let name = self.default_sink_name.as_ref()?;

        let id = self.sink_name_to_id.get(name)?;

        let node = self.nodes.get(id)?;
        Some(node)
    }
}
