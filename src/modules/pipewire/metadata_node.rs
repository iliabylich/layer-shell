use crate::modules::pipewire::Store;
use anyhow::{Context as _, Result};
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

        Store::register_meta(metadata_id, metadata).context("failed to register meta")?;
        Store::register_listener(metadata_id, Box::new(listener))
            .context("failed to register listener")?;

        Ok(())
    }

    fn prop_changed(key: &str, value: &str) -> i32 {
        if key == "default.audio.sink" {
            #[derive(serde::Deserialize)]
            struct Value {
                name: String,
            }
            if let Ok(Value { name }) = serde_json::from_str(value) {
                if let Err(err) = Store::register_default_sink_name(name) {
                    log::error!("failed to register default sink name: {:?}", err);
                }
            }
        }
        0
    }
}
