use std::collections::HashMap;

#[derive(Debug)]
pub(crate) enum Request {
    Get {
        path: String,
        headers: HashMap<String, String>,
    },
}

impl Request {
    pub(crate) fn get(path: impl Into<String>) -> Self {
        Self::Get {
            path: path.into(),
            headers: HashMap::new(),
        }
    }

    pub(crate) fn add_header(&mut self, name: impl Into<String>, value: impl Into<String>) {
        match self {
            Request::Get { headers, .. } => {
                headers.insert(name.into(), value.into());
            }
        }
    }

    pub(crate) fn into_bytes(self) -> Vec<u8> {
        match self {
            Request::Get { path, headers } => {
                let headers = headers
                    .into_iter()
                    .map(|(name, value)| format!("{name}: {value}"))
                    .collect::<Vec<_>>()
                    .join("\r\n");
                format!("GET {path} HTTP/1.1\r\n{headers}\r\n\r\n").into_bytes()
            }
        }
    }
}
