use anyhow::{Context, Result};
use pipewire::spa::{
    param::ParamType,
    pod::{serialize::PodSerializer, Object, Pod, Property, PropertyFlags, Value, ValueArray},
    sys::{
        SPA_PARAM_ROUTE_device, SPA_PARAM_ROUTE_index, SPA_PARAM_ROUTE_props, SPA_PARAM_Route,
        SPA_PROP_channelVolumes, SPA_PROP_mute,
    },
};

pub(crate) fn set_volume(volume: f32) {
    call_pw(Some(volume), None);
}

pub(crate) fn set_muted(muted: bool) {
    call_pw(None, Some(muted));
}

fn call_pw(volume: Option<f32>, muted: Option<bool>) {
    if let Err(err) = try_call_pw(volume, muted) {
        log::error!("failed to call PW: {:?}", err);
    }
}

fn try_call_pw(volume: Option<f32>, muted: Option<bool>) -> Result<()> {
    let store = crate::modules::pipewire::STORE::get();
    let (device, route) = store.default_device().context("no default device")?;

    let mut props = vec![];

    if let Some(volume) = volume {
        props.push(Property {
            key: SPA_PROP_channelVolumes,
            flags: PropertyFlags::empty(),
            value: Value::ValueArray(ValueArray::Float(vec![volume, volume])),
        });
    }

    if let Some(muted) = muted {
        props.push(Property {
            key: SPA_PROP_mute,
            flags: PropertyFlags::empty(),
            value: Value::Bool(muted),
        });
    }

    let values: Vec<u8> = PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &Value::Object(Object {
            type_: pipewire::spa::utils::SpaTypes::ObjectParamRoute.as_raw(),
            id: SPA_PARAM_Route,
            properties: vec![
                Property {
                    key: SPA_PARAM_ROUTE_index,
                    flags: PropertyFlags::empty(),
                    value: Value::Int(route.0),
                },
                Property {
                    key: SPA_PARAM_ROUTE_device,
                    flags: PropertyFlags::empty(),
                    value: Value::Int(route.1),
                },
                Property {
                    key: SPA_PARAM_ROUTE_props,
                    flags: PropertyFlags::empty(),
                    value: Value::Object(Object {
                        type_: pipewire::spa::utils::SpaTypes::ObjectParamProps.as_raw(),
                        id: SPA_PARAM_Route,
                        properties: props,
                    }),
                },
            ],
        }),
    )
    .context("invalid pod value")?
    .0
    .into_inner();
    let param = Pod::from_bytes(&values).context("invalid pod value")?;
    device.set_param(ParamType::Route, 0, param);
    Ok(())
}
