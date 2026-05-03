#[derive(Debug)]
pub(crate) enum HttpRequest {
    Get { host: &'static str, path: String },
}

impl HttpRequest {
    pub(crate) const fn get(host: &'static str, path: String) -> Self {
        Self::Get { host, path }
    }

    pub(crate) const fn host(&self) -> &'static str {
        match self {
            Self::Get { host, .. } => host,
        }
    }

    pub(crate) fn into_bytes(self) -> Vec<u8> {
        match self {
            Self::Get { path, host } => {
                format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n")
                    .into_bytes()
            }
        }
    }
}
