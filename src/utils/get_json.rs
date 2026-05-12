macro_rules! get_json {
    ($json:expr, $key:expr, $f:ident) => {
        $json
            .get($key)
            .with_context(|| format!("no {}", $key))?
            .$f()
            .with_context(|| format!("{} can't be coerced with {}", $key, stringify!($f)))?
    };

    ($json:expr, $key:expr) => {
        $json.get($key).with_context(|| format!("no {}", $key))?
    };
}
pub(crate) use get_json;
