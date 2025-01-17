use std::sync::LazyLock;

use crate::{lock_channel::LockChannel, modules::pipewire::Store};
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
    InternalCommand::SetVolume(volume).send();
}

pub(crate) fn set_muted(muted: bool) {
    InternalCommand::SetMuted(muted).send();
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum InternalCommand {
    SetVolume(f32),
    SetMuted(bool),
}

static CHANNEL: LazyLock<LockChannel<InternalCommand>> = LazyLock::new(LockChannel::new);

impl InternalCommand {
    pub(crate) fn send(self) {
        CHANNEL.emit(self);
    }

    pub(crate) fn try_recv() -> Option<Self> {
        CHANNEL.try_recv()
    }

    pub(crate) fn exec(self) {
        if let Err(err) = self.try_exec() {
            log::error!("failed to call PW: {:?}", err);
        }
    }

    fn try_exec(self) -> Result<()> {
        match self {
            Self::SetVolume(volume) => try_call_pw(Some(volume), None),
            Self::SetMuted(muted) => try_call_pw(None, Some(muted)),
        }
    }
}

fn try_call_pw(volume: Option<f32>, muted: Option<bool>) -> Result<()> {
    Store::with_default_device(|device, route| {
        let mut props = vec![];

        if let Some(volume) = volume {
            // convert to cubic
            let volume = volume.powf(3.0);
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
    })
}
