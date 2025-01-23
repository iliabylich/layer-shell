use anyhow::{Context, Result};
use pipewire::{device::Device, metadata::Metadata, node::Node, proxy::Listener};
use std::{cell::RefCell, collections::HashMap};

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

thread_local! {
    static STORE: RefCell<Option<Store>> = const { RefCell::new(None) };
}

fn with_store<T>(f: impl FnOnce(&mut Store) -> Result<T>) -> Result<T> {
    STORE.with(|store| {
        let mut store = store.borrow_mut();
        let store = store.as_mut().context("no PW store, are you in PW loop?")?;
        f(store)
    })
}

impl Store {
    pub(crate) fn init_for_current_thread() {
        STORE.with(|store| {
            let mut store = store.borrow_mut();
            *store = Some(Self {
                default_sink_name: None,

                nodes: HashMap::new(),
                meta: HashMap::new(),
                devices: HashMap::new(),

                listeners: HashMap::new(),

                sink_name_to_sink_id: HashMap::new(),
                sink_id_to_device_id: HashMap::new(),

                device_id_to_route: HashMap::new(),
            });
        });
    }

    pub(crate) fn register_meta(id: u32, meta: Metadata) -> Result<()> {
        with_store(|store| {
            store.meta.insert(id, meta);
            Ok(())
        })
    }

    pub(crate) fn register_device(id: u32, device: Device) -> Result<()> {
        with_store(|store| {
            store.devices.insert(id, device);
            Ok(())
        })
    }

    pub(crate) fn register_listener(id: u32, listener: Box<dyn Listener>) -> Result<()> {
        with_store(|store| {
            store.listeners.entry(id).or_default().push(listener);
            Ok(())
        })
    }

    pub(crate) fn register_sink(
        sink_id: u32,
        name: impl AsRef<str>,
        device_id: u32,
        sink: Node,
    ) -> Result<()> {
        with_store(|store| {
            store.nodes.insert(sink_id, sink);
            let name = name.as_ref().to_string();
            store.sink_name_to_sink_id.insert(name, sink_id);
            store.sink_id_to_device_id.insert(sink_id, device_id);
            Ok(())
        })
    }

    pub(crate) fn register_default_sink_name(name: String) -> Result<()> {
        with_store(|store| {
            store.default_sink_name = Some(name);
            Ok(())
        })
    }

    pub(crate) fn register_route(device_id: u32, route: (i32, i32)) -> Result<()> {
        with_store(|store| {
            store.device_id_to_route.insert(device_id, route);
            Ok(())
        })
    }

    pub(crate) fn with_default_device<T>(
        f: impl Fn(&Device, (i32, i32)) -> Result<T>,
    ) -> Result<T> {
        with_store(|store| {
            let sink_name = store
                .default_sink_name
                .as_ref()
                .context("no default sink name")?;
            let sink_id = store
                .sink_name_to_sink_id
                .get(sink_name)
                .context("no default sink ID")?;
            let device_id = store
                .sink_id_to_device_id
                .get(sink_id)
                .context("no default device ID")?;
            let device = store.devices.get(device_id).context("not default device")?;
            let route = store
                .device_id_to_route
                .get(device_id)
                .context("no default route")?;

            f(device, *route)
        })
    }

    pub(crate) fn remove(id: u32) -> Result<()> {
        with_store(|store| {
            store.devices.remove(&id);
            store.meta.remove(&id);
            store.nodes.remove(&id);
            store.listeners.remove(&id);
            Ok(())
        })
    }
}
