use crate::store::Store;
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

#[derive(Clone, Copy, Debug)]
pub enum Command {
    SetMuted(bool),
    SetVolume(f32),
}

impl Command {
    pub(crate) fn dispatch(self, store: &Store) {
        if let Err(err) = self.try_dispatch(store) {
            log::error!("failed to change sink node: {:?}", err);
        }
    }

    fn try_dispatch(self, store: &Store) -> Result<()> {
        let sink = store.default_sink().context("no default sink")?;
        match self {
            Command::SetMuted(muted) => set_muted(sink, muted),
            Command::SetVolume(volume) => set_volume(sink, volume),
        }
    }
}

fn set_muted(sink: Rc<Node>, muted: bool) -> Result<()> {
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

fn set_volume(sink: Rc<Node>, volume: f32) -> Result<()> {
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
