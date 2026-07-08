use crate::{sansio::HttpResponse, utils::get_json};
use alloc::vec::Vec;
use anyhow::{Context as _, Result, bail, ensure};
use jzon::JsonValue;

pub(crate) struct LocationResponse;

impl LocationResponse {
    pub(crate) fn parse(response: &HttpResponse) -> Result<(f64, f64)> {
        ensure!(response.status == 200);

        let json = jzon::parse(&response.body)?;
        let response = Response::from_json(&json).context("malformed JSON response")?;

        let get = |source: Source| -> Option<(f64, f64)> {
            response
                .location
                .iter()
                .find(|loc| loc.source == source)
                .map(|loc| (loc.lat, loc.lng))
        };
        let (lat, lng) = get(Source::FreeGeoIP)
            .or_else(|| get(Source::IpAPI))
            .or_else(|| get(Source::IpWhoIs))
            .context("failed to get at least one location")?;

        Ok((lat, lng))
    }
}

#[derive(Debug)]
struct Response {
    location: Vec<Location>,
}

impl Response {
    fn from_json(json: &JsonValue) -> Result<Self> {
        Ok(Self {
            location: get_json!(json, "location", as_array)
                .iter()
                .map(Location::from_json)
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

#[derive(Debug)]
struct Location {
    lat: f64,
    lng: f64,
    source: Source,
}

impl Location {
    fn from_json(json: &JsonValue) -> Result<Self> {
        let lat = get_json!(json, "lat", as_f64);
        let lng = get_json!(json, "lng", as_f64);
        let source = Source::from_str(get_json!(json, "source", as_str))?;
        Ok(Self { lat, lng, source })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Source {
    FreeGeoIP,
    IpAPI,
    IpWhoIs,
}

impl Source {
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "freegeoip" => Ok(Self::FreeGeoIP),
            "ipapi" => Ok(Self::IpAPI),
            "ipwhois" => Ok(Self::IpWhoIs),
            _ => bail!("unknown source {s:?}"),
        }
    }
}
