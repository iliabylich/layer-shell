use pipewire::spa::{
    pod::{deserialize::PodDeserializer, Pod, Value, ValueArray},
    sys::SPA_PROP_channelVolumes,
};

pub(crate) fn try_parse_volume_changed_event(param: Option<&Pod>) -> Option<Vec<f32>> {
    let param = param?;
    let (_, data) = PodDeserializer::deserialize_any_from(param.as_bytes()).ok()?;
    if let Value::Object(obj) = data {
        for prop in obj.properties {
            if prop.key == SPA_PROP_channelVolumes {
                if let Value::ValueArray(ValueArray::Float(floats)) = prop.value {
                    return Some(floats);
                }
            }
        }
    }

    None
}
