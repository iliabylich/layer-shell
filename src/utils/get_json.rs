macro_rules! get_json {
    ($json:expr, $key:expr, $f:ident) => {
        $json
            .get_key_value($key)
            .map_err(|_| anyhow::anyhow!(concat!("no ", $key)))?
            .$f()
            .map_err(|_| {
                anyhow::anyhow!(concat!($key, " can't be coerced with ", stringify!($f)))
            })?
    };

    ($json:expr, $key:expr) => {
        $json
            .get_key_value($key)
            .map_err(|_| anyhow::anyhow!(concat!("no ", $key)))?
    };
}
pub(crate) use get_json;
