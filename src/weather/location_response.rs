use crate::https::Response;
use anyhow::{Context as _, Result, ensure};
use serde::Deserialize;

pub(crate) struct LocationResponse;

impl LocationResponse {
    pub(crate) fn parse(response: Response) -> Result<(f64, f64)> {
        ensure!(response.status == 200);

        #[derive(Debug, Deserialize)]
        struct Response {
            location: Vec<Location>,
        }
        #[derive(Debug, Deserialize)]
        struct Location {
            lat: f64,
            lng: f64,
            source: Source,
        }
        #[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
        enum Source {
            #[serde(rename = "freegeoip")]
            FreeGeoIP,
            #[serde(rename = "ipapi")]
            IpAPI,
            #[serde(rename = "ipwhois")]
            IpWhoIs,
        }

        let response: Response =
            serde_json::from_str(&response.body).context("malformed JSON output")?;

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
