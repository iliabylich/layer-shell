use pipewire::{
    registry::GlobalObject,
    spa::{
        pod::{deserialize::PodDeserializer, Pod, Value, ValueArray},
        sys::SPA_PROP_channelVolumes,
        utils::dict::DictRef,
    },
};

pub(crate) fn parse_name(obj: &GlobalObject<&DictRef>) -> Option<String> {
    let props = obj.props?;
    let media_class = props.get("media.class")?;
    let node_name = props.get("node.name")?;

    if media_class == "Audio/Sink" {
        return Some(node_name.to_string());
    }

    None
}

pub(crate) fn parse_volume_changed_event(param: Option<&Pod>) -> Option<Vec<f32>> {
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
