use crate::scheduler::Module;
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
use std::{any::Any, time::Duration};

mod audio_device;
mod audio_sink;
mod command;
mod metadata_node;
mod store;

use command::InternalCommand;
use store::Store;

pub(crate) struct Pipewire;

impl Pipewire {
    pub(crate) fn set_muted(muted: bool) -> Result<()> {
        command::set_muted(muted)
    }

    pub(crate) fn set_volume(volume: f32) -> Result<()> {
        command::set_volume(volume)
    }
}

impl Module for Pipewire {
    const NAME: &str = "Pipewire";
    const INTERVAL: Option<u64> = None;

    fn start() -> Result<Box<dyn Any + Send + 'static>> {
        std::thread::spawn(|| {
            if let Err(err) = start_pw_mainloop() {
                log::error!("{}", err);
            }
        });

        Ok(Box::new(0))
    }

    fn tick(_: &mut Box<dyn Any + Send + 'static>) -> Result<()> {
        unreachable!()
    }
}

fn start_pw_mainloop() -> Result<()> {
    let mainloop = MainLoop::new(None).context("failed to instantiate PW loop")?;
    let context = Context::new(&mainloop)?;
    let core = context.connect(None)?;

    Store::init_for_current_thread();
    let registry: &'static Registry = Box::leak(Box::new(core.get_registry()?));

    let _listener = start_pw_listener(registry);

    let timer = mainloop.loop_().add_timer(|_| {
        while let Some(cmd) = InternalCommand::try_recv() {
            if let Err(err) = cmd.exec() {
                log::error!("Failed to call PW: {:?}", err);
            }
        }
    });

    timer
        .update_timer(
            Some(Duration::from_millis(100)),
            Some(Duration::from_millis(100)),
        )
        .into_result()
        .context("invalid timer")?;

    mainloop.run();

    Ok(())
}

fn start_pw_listener(registry: &'static Registry) -> Listener {
    registry
        .add_listener_local()
        .global(|obj| {
            if let Err(err) = on_global_object_added(registry, obj) {
                log::error!("Failed to track new global object: {:?}", err);
            }
        })
        .global_remove(on_global_object_removed)
        .register()
}

fn on_global_object_added(registry: &Registry, obj: &GlobalObject<&DictRef>) -> Result<()> {
    let Some(props) = obj.props else {
        // ignore empty objects
        return Ok(());
    };

    if props.get("metadata.name") == Some("default") {
        let metadata: Metadata = registry.bind(obj).context("not a Metadata")?;
        metadata_node::MetadataNode::added(obj.id, metadata)?;
    }

    if props.get("media.class") == Some("Audio/Device") {
        let device: Device = registry.bind(obj).context("not a Device")?;
        audio_device::AudioDevice::added(obj.id, device)?;
    }

    if props.get("media.class") == Some("Audio/Sink") {
        let node: Node = registry.bind(obj).context("not a Node")?;
        audio_sink::AudioSink::added(obj.id, props, node)?;
    }

    Ok(())
}

fn on_global_object_removed(id: u32) {
    if let Err(err) = Store::remove(id) {
        log::error!("Failed to remove device: {:?}", err);
    }
}
