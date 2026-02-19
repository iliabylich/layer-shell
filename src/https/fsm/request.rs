#[derive(Debug)]
pub(crate) enum Request {
    Get { host: &'static str, path: String },
}

impl Request {
    pub(crate) fn get(host: &'static str, path: impl Into<String>) -> Self {
        Self::Get {
            host,
            path: path.into(),
        }
    }

    pub(crate) fn into_bytes(self) -> Vec<u8> {
        match self {
            Request::Get { path, host } => {
                format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n")
                    .into_bytes()
            }
        }
    }
}
