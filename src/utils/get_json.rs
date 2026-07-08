macro_rules! get_json {
    ($json:expr, $key:expr, $f:ident) => {
        $json
            .get($key)
            .context(concat!("no ", $key))?
            .$f()
            .context(concat!($key, " can't be coerced with ", stringify!($f)))?
    };

    ($json:expr, $key:expr) => {
        $json.get($key).context(concat!("no ", $key))?
    };
}
pub(crate) use get_json;
