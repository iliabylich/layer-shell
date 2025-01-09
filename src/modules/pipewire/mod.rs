use anyhow::{bail, Context as _, Result};
use pipewire::{
    context::Context,
    device::Device,
    main_loop::MainLoop,
    metadata::Metadata,
    node::Node,
    registry::{GlobalObject, Listener, Registry},
    spa::{
        param::ParamType,
        pod::{deserialize::PodDeserializer, Pod, Value, ValueArray},
        sys::{
            SPA_PARAM_ROUTE_device, SPA_PARAM_ROUTE_index, SPA_PROP_channelVolumes, SPA_PROP_mute,
        },
        utils::dict::DictRef,
    },
};

mod command;
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
        .global(|obj| {
            if let Err(err) = on_global_object_added(obj) {
                log::error!("Failed to track new global object: {:?}", err);
            }
        })
        .global_remove(on_global_object_removed)
        .register()
}

fn on_global_object_added(obj: &GlobalObject<&DictRef>) -> Result<()> {
    let Some(props) = obj.props else {
        // ignore empty objects
        return Ok(());
    };

    if props.get("metadata.name") == Some("default") {
        let metadata = REGISTRY::get()
            .bind::<Metadata, _>(obj)
            .context("not a Metadata")?;
        on_metadata_node_added(obj.id, metadata)?;
    }

    if props.get("media.class") == Some("Audio/Device") {
        let device = REGISTRY::get()
            .bind::<Device, _>(obj)
            .context("not a Device")?;
        on_audio_device_added(obj.id, device)?;
    }

    if props.get("media.class") == Some("Audio/Sink") {
        let node = REGISTRY::get().bind::<Node, _>(obj).context("not a Node")?;
        on_audio_sink_added(obj.id, props, node)?;
    }

    Ok(())
}

fn on_global_object_removed(id: u32) {
    STORE::get().remove(id);
}

fn on_metadata_node_added(metadata_id: u32, metadata: Metadata) -> Result<()> {
    let listener = metadata
        .add_listener_local()
        .property(|_, key, _, value| {
            if let (Some(key), Some(value)) = (key, value) {
                on_metadata_node_changed(key, value)
            } else {
                0
            }
        })
        .register();

    STORE::get().register_meta(metadata_id, metadata);
    STORE::get().register_listener(metadata_id, Box::new(listener));

    Ok(())
}

fn on_metadata_node_changed(key: &str, value: &str) -> i32 {
    if key == "default.audio.sink" {
        #[derive(serde::Deserialize)]
        struct Value {
            name: String,
        }
        if let Ok(Value { name }) = serde_json::from_str(value) {
            STORE::get().register_default_sink_name(name);
        }
    }
    0
}

fn on_audio_device_added(device_id: u32, device: Device) -> Result<()> {
    device.subscribe_params(&[ParamType::Route]);
    let listener = device
        .add_listener_local()
        .param(move |_, _, _, _, param| {
            if let Some(param) = param {
                if let Err(err) = on_audio_device_route_changed(device_id, param) {
                    log::error!("Failed to track route change: {:?}", err);
                }
            } else {
                // ignore
            }
        })
        .register();

    STORE::get().register_device(device_id, device);
    STORE::get().register_listener(device_id, Box::new(listener));

    Ok(())
}

fn on_audio_device_route_changed(device_id: u32, param: &Pod) -> Result<()> {
    let value = match PodDeserializer::deserialize_any_from(param.as_bytes()) {
        Ok((_, value)) => value,
        Err(err) => bail!("Failed to parse sink node's route param: {:?}", err),
    };

    let Value::Object(object) = value else {
        bail!("Pod value is not an Object");
    };

    let mut route_index = None;
    let mut route_device = None;
    for prop in object.properties {
        if prop.key == SPA_PARAM_ROUTE_index {
            let Value::Int(int) = prop.value else {
                bail!("Route index is not an Int");
            };

            route_index = Some(int);
        }

        if prop.key == SPA_PARAM_ROUTE_device {
            let Value::Int(int) = prop.value else {
                bail!("Route device is not an Int");
            };
            route_device = Some(int);
        }
    }

    let route_index = route_index.context("no Route index prop")?;
    let route_device = route_device.context("no Route device prop")?;

    STORE::get().register_route(device_id, (route_index, route_device));

    Ok(())
}

fn on_audio_sink_added(id: u32, props: &DictRef, node: Node) -> Result<()> {
    let sink_name = props.get("node.name").context("no sink.name")?;
    let device_id = props
        .get("device.id")
        .context("no device.id")?
        .parse::<u32>()
        .context("device.id is not a number")?;

    node.subscribe_params(&[ParamType::Props]);
    let listener = node
        .add_listener_local()
        .param(|_, _, _, _, param| {
            if let Some(param) = param {
                if let Err(err) = on_sink_node_prop_changed(param) {
                    log::error!("Failed to track sink prop change: {:?}", err);
                }
            } else {
                // ignore
            }
        })
        .register();

    STORE::get().register_sink(id, sink_name, device_id, node);
    STORE::get().register_listener(id, Box::new(listener));

    Ok(())
}

fn on_sink_node_prop_changed(param: &Pod) -> Result<()> {
    let value = match PodDeserializer::deserialize_any_from(param.as_bytes()) {
        Ok((_, value)) => value,
        Err(err) => bail!("Failed to parse sink node's route param: {:?}", err),
    };

    let Value::Object(object) = value else {
        bail!("Pod is not an Object");
    };

    for prop in object.properties {
        if prop.key == SPA_PROP_channelVolumes {
            if let Value::ValueArray(ValueArray::Float(floats)) = prop.value {
                if floats.len() == 2 {
                    let volume = (floats[0] + floats[1]) / 2.0;
                    let event = Event::Volume { volume };
                    event.emit();
                } else {
                    bail!("channelVolumes must contain exactly two elements");
                }
            } else {
                bail!("channelVolumes must be an Array of Floats");
            }
        } else if prop.key == SPA_PROP_mute {
            if let Value::Bool(bool) = prop.value {
                let event = Event::Mute { muted: bool };
                event.emit();
            } else {
                bail!("mute must be Bool");
            }
        }
    }

    Ok(())
}
