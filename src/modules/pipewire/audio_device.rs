use crate::modules::pipewire::Store;
use anyhow::{bail, Context as _, Result};
use pipewire::{
    device::Device,
    spa::{
        param::ParamType,
        pod::{deserialize::PodDeserializer, Pod, Value},
        sys::{SPA_PARAM_ROUTE_device, SPA_PARAM_ROUTE_index},
    },
};

pub(crate) struct AudioDevice;

impl AudioDevice {
    pub(crate) fn added(device_id: u32, device: Device) -> Result<()> {
        device.subscribe_params(&[ParamType::Route]);
        let listener = device
            .add_listener_local()
            .param(move |_, _, _, _, param| {
                if let Some(param) = param {
                    if let Err(err) = Self::route_changed(device_id, param) {
                        log::error!("Failed to track route change: {:?}", err);
                    }
                } else {
                    // ignore
                }
            })
            .register();

        Store::register_device(device_id, device).context("failed to register device")?;
        Store::register_listener(device_id, Box::new(listener))
            .context("failed to register listener")?;

        Ok(())
    }

    fn route_changed(device_id: u32, param: &Pod) -> Result<()> {
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

        Store::register_route(device_id, (route_index, route_device))
            .context("failed to register route")?;

        Ok(())
    }
}
