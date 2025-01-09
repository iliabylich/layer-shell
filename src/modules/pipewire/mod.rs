use anyhow::{Context as _, Result};
use pipewire::{
    context::Context,
    device::Device,
    main_loop::MainLoop,
    metadata::Metadata,
    node::Node,
    registry::{GlobalObject, Listener, Registry},
    spa::utils::dict::DictRef,
};

mod audio_device;
mod audio_sink;
mod command;
mod metadata_node;
mod store;

use store::Store;

use crate::global;

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
        let metadata: Metadata = REGISTRY::get().bind(obj).context("not a Metadata")?;
        metadata_node::MetadataNode::on_add(obj.id, metadata)?;
    }

    if props.get("media.class") == Some("Audio/Device") {
        let device: Device = REGISTRY::get().bind(obj).context("not a Device")?;
        audio_device::AudioDevice::on_add(obj.id, device)?;
    }

    if props.get("media.class") == Some("Audio/Sink") {
        let node: Node = REGISTRY::get().bind(obj).context("not a Node")?;
        audio_sink::AudioSink::on_add(obj.id, props, node)?;
    }

    Ok(())
}

fn on_global_object_removed(id: u32) {
    STORE::get().remove(id);
}
