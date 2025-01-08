use anyhow::{Context as _, Result};
use pipewire::{
    context::Context,
    main_loop::MainLoop,
    metadata::Metadata,
    node::Node,
    registry::{GlobalObject, Listener, Registry},
    spa::{param::ParamType, pod::Pod, utils::dict::DictRef},
};

mod command;
mod sink;
mod store;

use store::Store;

use crate::{global, Event};

pub(crate) use command::set_volume;

pub(crate) fn setup() {
    std::thread::spawn(move || {
        if let Err(err) = start_pw_mainloop() {
            log::error!("{}", err);
        }
    });
}

global!(STORE, Store);
global!(REGISTRY, Registry);

fn start_pw_mainloop() -> Result<()> {
    let mainloop = MainLoop::new(None).context("failed to instantiate PW loop")?;
    let context = Context::new(&mainloop)?;
    let core = context.connect(None)?;

    REGISTRY::set(core.get_registry()?);
    STORE::set(Store::new());

    let _listener = start_pw_listener();

    mainloop.run();

    Ok(())
}

fn start_pw_listener() -> Listener {
    REGISTRY::get()
        .add_listener_local()
        .global(on_global_object_added)
        .register()
}

fn on_global_object_added(obj: &GlobalObject<&DictRef>) {
    let Some(props) = obj.props else {
        return;
    };

    if props.get("metadata.name") == Some("default") {
        on_default_node_added(obj);
    }

    if props.get("media.class") == Some("Audio/Sink") {
        if let Some(name) = props.get("node.name") {
            on_sink_node_added(obj, name);
        }
    }
}

fn on_default_node_added(obj: &GlobalObject<&DictRef>) {
    let Ok(meta) = REGISTRY::get().bind::<Metadata, _>(obj) else {
        log::error!("failed to bind to Metadata object");
        return;
    };

    let listener = meta
        .add_listener_local()
        .property(on_default_node_attribute_changed)
        .register();

    STORE::get().add_meta(obj.id, meta);
    STORE::get().add_listener(obj.id, Box::new(listener));
}

fn on_default_node_attribute_changed(
    _: u32,
    key: Option<&str>,
    _: Option<&str>,
    value: Option<&str>,
) -> i32 {
    if let (Some("default.audio.sink"), Some(value)) = (key, value) {
        #[derive(serde::Deserialize)]
        struct Value {
            name: String,
        }
        if let Ok(Value { name }) = serde_json::from_str(value) {
            STORE::get().set_default_sink_name(name);
        }
    }
    0
}

fn on_sink_node_added(obj: &GlobalObject<&DictRef>, name: &str) {
    let Ok(node) = REGISTRY::get().bind::<Node, _>(obj) else {
        log::error!("failed to bind to Metadata object");
        return;
    };

    node.subscribe_params(&[ParamType::Props]);
    let listener = node
        .add_listener_local()
        .param(on_sink_node_prop_changed)
        .register();

    STORE::get().add_node(obj.id, node);
    STORE::get().add_listener(obj.id, Box::new(listener));
    STORE::get().add_sink_name(name, obj.id);
}

fn on_sink_node_prop_changed(_: i32, _: ParamType, _: u32, _: u32, param: Option<&Pod>) {
    if let Some(channels) = sink::try_parse_volume_changed_event(param) {
        let volume = channels[0];
        let event = Event::Volume { volume };
        event.emit();
    }
}
