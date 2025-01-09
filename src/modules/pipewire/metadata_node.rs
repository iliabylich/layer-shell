use crate::modules::pipewire::STORE;
use anyhow::Result;
use pipewire::metadata::Metadata;

pub(crate) struct MetadataNode;

impl MetadataNode {
    pub(crate) fn added(metadata_id: u32, metadata: Metadata) -> Result<()> {
        let listener = metadata
            .add_listener_local()
            .property(|_, key, _, value| {
                if let (Some(key), Some(value)) = (key, value) {
                    Self::prop_changed(key, value)
                } else {
                    0
                }
            })
            .register();

        STORE::get().register_meta(metadata_id, metadata);
        STORE::get().register_listener(metadata_id, Box::new(listener));

        Ok(())
    }

    fn prop_changed(key: &str, value: &str) -> i32 {
        if key == "default.audio.sink" {
            #[derive(serde::Deserialize)]
            struct Value {
                name: String,
            }
            if let Ok(Value { name }) = serde_json::from_str(value) {
                STORE::get().register_default_sink_name(name);
            }
        }
        0
    }
}
