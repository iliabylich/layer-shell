use crate::{sansio::HttpResponse, utils::get_json};
use alloc::vec::Vec;
use anyhow::{Context as _, Result, bail, ensure};
use microjson::JSONValue;

pub(crate) struct LocationResponse;

impl LocationResponse {
    pub(crate) fn parse(response: &HttpResponse) -> Result<(f64, f64)> {
        ensure!(response.status == 200);

        let json = JSONValue::load(&response.body);
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
    fn from_json(json: &JSONValue<'_>) -> Result<Self> {
        Ok(Self {
            location: get_json!(json, "location", iter_array)
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
    fn from_json(json: JSONValue) -> Result<Self> {
        let lat = f64::from(get_json!(json, "lat", read_float));
        let lng = f64::from(get_json!(json, "lng", read_float));
        let source = Source::from_str(get_json!(json, "source", read_string))?;
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
