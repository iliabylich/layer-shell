use crate::utils::ArrayWriter;
use alloc::{string::String, vec::Vec};
use anyhow::{Context as _, Result};
use core::{ffi::CStr, fmt::Write};

#[derive(Debug)]
pub(crate) enum HttpRequest {
    Get { host: &'static CStr, path: String },
}

impl HttpRequest {
    pub(crate) const fn get(host: &'static CStr, path: String) -> Self {
        Self::Get { host, path }
    }

    pub(crate) const fn host(&self) -> &'static CStr {
        match self {
            Self::Get { host, .. } => host,
        }
    }

    pub(crate) fn into_bytes(self) -> Result<Vec<u8>> {
        match self {
            Self::Get { path, host } => {
                let mut buf = [0; 1_024];
                let mut w = ArrayWriter::new(&mut buf);
                let host = host.to_str().context("non-utf8 host")?;
                write!(
                    &mut w,
                    "GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
                )?;
                Ok(w.as_bytes()?.to_vec())
            }
        }
    }
}
