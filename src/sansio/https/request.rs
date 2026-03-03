#[derive(Debug)]
pub(crate) enum HttpsRequest {
    Get { host: &'static str, path: String },
}

impl HttpsRequest {
    pub(crate) fn get(host: &'static str, path: impl Into<String>) -> Self {
        Self::Get {
            host,
            path: path.into(),
        }
    }

    pub(crate) fn host(&self) -> &'static str {
        match self {
            HttpsRequest::Get { host, .. } => host,
        }
    }

    pub(crate) fn into_bytes(self) -> Vec<u8> {
        match self {
            HttpsRequest::Get { path, host } => {
                format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n")
                    .into_bytes()
            }
        }
    }
}
