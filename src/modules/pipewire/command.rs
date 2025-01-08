use anyhow::{Context, Result};
use pipewire::{
    node::Node,
    spa::{
        param::ParamType,
        pod::{serialize::PodSerializer, Object, Pod, Property, PropertyFlags, Value, ValueArray},
        sys::{SPA_PARAM_Props, SPA_PROP_channelVolumes, SPA_PROP_mute, SPA_PROP_volume},
    },
};
use std::rc::Rc;

pub(crate) fn set_volume(volume: f32) {
    if let Err(err) = try_set_volume(volume) {
        log::error!("failed to call PW: {:?}", err);
    }
}

fn try_set_volume(volume: f32) -> Result<()> {
    let store = crate::modules::pipewire::STORE::get();
    let sink = store.default_sink().context("no default sink")?;

    set_volume_attribute_on_node(sink, volume)
}

fn set_volume_attribute_on_node(sink: &Node, volume: f32) -> Result<()> {
    let values: Vec<u8> = PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &Value::Object(Object {
            type_: pipewire::spa::utils::SpaTypes::ObjectParamProps.as_raw(),
            id: SPA_PARAM_Props,
            properties: vec![
                Property {
                    key: SPA_PROP_volume,
                    flags: PropertyFlags::empty(),
                    value: Value::Float(volume),
                },
                Property {
                    key: SPA_PROP_channelVolumes,
                    flags: PropertyFlags::empty(),
                    value: Value::ValueArray(ValueArray::Float(vec![volume, volume])),
                },
            ],
        }),
    )
    .context("invalid pod value")?
    .0
    .into_inner();
    let param = Pod::from_bytes(&values).context("invalid pod value")?;
    sink.set_param(ParamType::Props, 0, param);
    Ok(())
}

#[allow(dead_code)]
fn set_muted_attribute_on_node(sink: Rc<Node>, muted: bool) -> Result<()> {
    let values: Vec<u8> = PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &Value::Object(Object {
            type_: pipewire::spa::utils::SpaTypes::ObjectParamProps.as_raw(),
            id: SPA_PARAM_Props,
            properties: vec![Property {
                key: SPA_PROP_mute,
                flags: PropertyFlags::empty(),
                value: Value::Bool(muted),
            }],
        }),
    )
    .context("invalid pod value")?
    .0
    .into_inner();
    let param = Pod::from_bytes(&values).context("invalid pod value")?;
    sink.set_param(ParamType::Props, 0, param);

    Ok(())
}
