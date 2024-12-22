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

#[derive(Debug)]
pub struct SetVolume(pub f64);

impl SetVolume {
    pub async fn exec(self) {
        if let Err(err) = crate::command_sender().send(self) {
            log::error!("Faied to send PW command to PW thread: {:?}", err);
        }
    }
}

impl SetVolume {
    pub(crate) fn dispatch_in_current_thread(self, store: &Store) {
        if let Err(err) = self.try_dispatch_in_current_thread(store) {
            log::error!("failed to change sink node: {:?}", err);
        }
    }

    fn try_dispatch_in_current_thread(self, store: &Store) -> Result<()> {
        let sink = store.default_sink().context("no default sink")?;
        set_volume(sink, self.0)
    }
}

#[allow(dead_code)]
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

fn set_volume(sink: Rc<Node>, volume: f64) -> Result<()> {
    let volume = volume as f32;

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
