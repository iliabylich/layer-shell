use anyhow::{Context as _, Result, bail};
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Response {
    pub(crate) status: u16,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: String,
}

impl Response {
    pub(crate) fn parse(data: Vec<u8>) -> Result<Self> {
        let data = String::from_utf8(data)?;
        let (pre, body) = data
            .split_once("\r\n\r\n")
            .context("no separator between headers and body")?;
        let (status, headers) = pre
            .split_once("\r\n")
            .context("no separator between status line and headers")?;

        let status = status
            .split(" ")
            .nth(1)
            .context("malformed status line")?
            .parse::<u16>()
            .context("non-numeric HTTP status")?;

        let headers = {
            let mut out = HashMap::new();
            for line in headers.split("\r\n") {
                let (name, value) = line.split_once(": ").context("malformed header")?;
                out.insert(name.to_string(), value.to_string());
            }
            out
        };

        let body = if headers
            .get("Transfer-Encoding")
            .is_some_and(|s| s == "chunked")
        {
            decode_chunked_body(body)?
        } else {
            body.to_string()
        };

        Ok(Self {
            status,
            headers,
            body,
        })
    }
}

fn decode_chunked_body(body: &str) -> Result<String> {
    let mut decoded = String::new();
    let mut rest = body;

    loop {
        let (chunk_size_hex, remainder) = rest
            .split_once("\r\n")
            .context("malformed chunked encoding: missing chunk size terminator")?;

        let chunk_size = usize::from_str_radix(chunk_size_hex.trim(), 16)
            .context("malformed chunked encoding: invalid chunk size")?;

        if chunk_size == 0 {
            break;
        }

        if remainder.len() < chunk_size {
            bail!("malformed chunked encoding: chunk data too short");
        }

        decoded.push_str(&remainder[..chunk_size]);

        rest = &remainder[chunk_size..];
        if !rest.starts_with("\r\n") {
            bail!("malformed chunked encoding: missing chunk data terminator");
        }
        rest = &rest[2..];
    }

    Ok(decoded)
}
