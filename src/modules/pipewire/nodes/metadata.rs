use pipewire::{registry::GlobalObject, spa::utils::dict::DictRef};

pub(crate) fn is_default(obj: &GlobalObject<&DictRef>) -> bool {
    try_parse_default(obj).is_some()
}

fn try_parse_default(obj: &GlobalObject<&DictRef>) -> Option<()> {
    let props = obj.props?;
    let name = props.get("metadata.name")?;
    if name == "default" {
        return Some(());
    }

    None
}

pub(crate) fn parse_audio_sink_changed(key: Option<&str>, value: Option<&str>) -> Option<String> {
    if let Some(("default.audio.sink", value)) = key.zip(value) {
        #[derive(serde::Deserialize)]
        struct Value {
            name: String,
        }
        let Value { name } = serde_json::from_str(value).ok()?;
        Some(name)
    } else {
        None
    }
}
