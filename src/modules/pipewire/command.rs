use anyhow::{Context, Result};
use pipewire::{
    device::Device,
    spa::{
        param::ParamType,
        pod::{serialize::PodSerializer, Object, Pod, Property, PropertyFlags, Value, ValueArray},
        sys::{
            SPA_PARAM_ROUTE_device, SPA_PARAM_ROUTE_index, SPA_PARAM_ROUTE_props, SPA_PARAM_Route,
            SPA_PROP_channelVolumes, SPA_PROP_mute,
        },
    },
};

pub(crate) fn set_volume(volume: f32) {
    if let Err(err) = try_set_volume(volume) {
        log::error!("failed to call PW: {:?}", err);
    }
}

fn try_set_volume(volume: f32) -> Result<()> {
    let store = crate::modules::pipewire::STORE::get();
    let (device, route) = store.default_device().context("no default device")?;

    call_pw(device, route, Some(volume), None)
}

fn call_pw(
    device: &Device,
    route: (i32, i32),
    volume: Option<f32>,
    mute: Option<bool>,
) -> Result<()> {
    let mut props = vec![];

    if let Some(volume) = volume {
        props.push(Property {
            key: SPA_PROP_channelVolumes,
            flags: PropertyFlags::empty(),
            value: Value::ValueArray(ValueArray::Float(vec![volume, volume])),
        });
    }

    if let Some(mute) = mute {
        props.push(Property {
            key: SPA_PROP_mute,
            flags: PropertyFlags::empty(),
            value: Value::Bool(mute),
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
