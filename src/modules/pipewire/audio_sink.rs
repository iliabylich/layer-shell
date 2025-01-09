use crate::{modules::pipewire::STORE, Event};
use anyhow::{bail, Context as _, Result};
use pipewire::{
    node::Node,
    spa::{
        param::ParamType,
        pod::{deserialize::PodDeserializer, Pod, Value, ValueArray},
        sys::{SPA_PROP_channelVolumes, SPA_PROP_mute},
        utils::dict::DictRef,
    },
};

pub(crate) struct AudioSink;

impl AudioSink {
    pub(crate) fn on_add(id: u32, props: &DictRef, node: Node) -> Result<()> {
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
                    if let Err(err) = Self::on_prop_change(param) {
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

    fn on_prop_change(param: &Pod) -> Result<()> {
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
}
