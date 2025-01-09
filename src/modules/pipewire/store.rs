use pipewire::{device::Device, metadata::Metadata, node::Node, proxy::Listener};
use std::collections::HashMap;

pub(crate) struct Store {
    default_sink_name: Option<String>,

    nodes: HashMap<u32, Node>,
    meta: HashMap<u32, Metadata>,
    devices: HashMap<u32, Device>,

    listeners: HashMap<u32, Vec<Box<dyn Listener>>>,

    sink_name_to_sink_id: HashMap<String, u32>,
    sink_id_to_device_id: HashMap<u32, u32>,

    device_id_to_route: HashMap<u32, (i32, i32)>,
}

impl Store {
    pub(crate) fn new() -> Self {
        Self {
            default_sink_name: None,

            nodes: HashMap::new(),
            meta: HashMap::new(),
            devices: HashMap::new(),

            listeners: HashMap::new(),

            sink_name_to_sink_id: HashMap::new(),
            sink_id_to_device_id: HashMap::new(),

            device_id_to_route: HashMap::new(),
        }
    }

    pub(crate) fn register_meta(&mut self, id: u32, meta: Metadata) {
        self.meta.insert(id, meta);
    }

    pub(crate) fn register_device(&mut self, id: u32, device: Device) {
        self.devices.insert(id, device);
    }

    pub(crate) fn register_listener(&mut self, id: u32, listener: Box<dyn Listener>) {
        self.listeners.entry(id).or_default().push(listener);
    }

    pub(crate) fn register_sink(
        &mut self,
        sink_id: u32,
        name: impl AsRef<str>,
        device_id: u32,
        sink: Node,
    ) {
        self.nodes.insert(sink_id, sink);
        let name = name.as_ref().to_string();
        self.sink_name_to_sink_id.insert(name, sink_id);
        self.sink_id_to_device_id.insert(sink_id, device_id);
    }

    pub(crate) fn register_default_sink_name(&mut self, name: String) {
        self.default_sink_name = Some(name);
    }

    pub(crate) fn register_route(&mut self, device_id: u32, route: (i32, i32)) {
        self.device_id_to_route.insert(device_id, route);
    }

    pub(crate) fn default_device(&self) -> Option<(&Device, (i32, i32))> {
        let sink_name = self.default_sink_name.as_ref()?;
        let sink_id = self.sink_name_to_sink_id.get(sink_name)?;
        let device_id = self.sink_id_to_device_id.get(sink_id)?;
        let device = self.devices.get(device_id)?;
        let route = self.device_id_to_route.get(device_id)?;
        Some((device, *route))
    }

    pub(crate) fn remove(&mut self, id: u32) {
        self.devices.remove(&id);
        self.meta.remove(&id);
        self.nodes.remove(&id);
        self.listeners.remove(&id);
    }
}
