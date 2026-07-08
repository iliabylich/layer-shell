use crate::utils::ArrayWriter;
use alloc::{string::String, vec::Vec};
use anyhow::Result;
use core::fmt::Write;

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

    pub(crate) fn into_bytes(self) -> Result<Vec<u8>> {
        match self {
            Self::Get { path, host } => {
                let mut buf = [0; 1_024];
                let mut w = ArrayWriter::new(&mut buf);
                write!(
                    &mut w,
                    "GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
                )?;
                Ok(w.as_bytes()?.to_vec())
            }
        }
    }
}
