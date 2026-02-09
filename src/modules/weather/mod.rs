use crate::{
    Event, dns::DnsResolver, https::HttpsConnection,
    modules::weather::weather_response::WeatherResponse, timerfd::Tick, user_data::ModuleId,
};
use libc::sockaddr_in;
pub use weather_code::WeatherCode;
pub use weather_response::{WeatherOnDay, WeatherOnHour};

mod weather_code;
mod weather_response;

const HOST: &str = "api.open-meteo.com";

pub(crate) struct Weather {
    lat: f64,
    lng: f64,
    dns: Box<DnsResolver>,
    address: Option<sockaddr_in>,
    https: Option<HttpsConnection>,
}

impl Weather {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            lat: 0.,
            lng: 0.,
            https: None,
            address: None,
            dns: DnsResolver::new(ModuleId::WeatherDNS, HOST.as_bytes()),
        })
    }

    pub(crate) fn init(&mut self, lat: f64, lng: f64) {
        self.lat = lat;
        self.lng = lng;
        self.dns.init();
    }

    fn request_weather(&mut self) {
        let Some(address) = self.address else {
            return;
        };

        let query = format!(
            "{}={}&{}={}&{}={}&{}={}&{}={}&{}={}",
            "latitude",
            self.lat,
            "longitude",
            self.lng,
            "current",
            "temperature_2m,weather_code",
            "hourly",
            "temperature_2m,weather_code",
            "daily",
            "temperature_2m_min,temperature_2m_max,weather_code",
            "timezone",
            "Europe/Warsaw"
        );

        let mut https = HttpsConnection::new(
            HOST,
            &format!("/v1/forecast?{query}"),
            ModuleId::WeatherHTTPS,
        );
        https.init(address, 443);
        self.https = Some(https);
    }

    pub(crate) fn process_dns(&mut self, op: u8, res: i32) {
        if let Some(address) = self.dns.process(op, res) {
            self.address = Some(address);
            self.request_weather();
        }
    }

    pub(crate) fn process_https(&mut self, op: u8, res: i32, events: &mut Vec<Event>) {
        let Some(https) = self.https.as_mut() else {
            return;
        };
        let Some(response) = https.process(op, res) else {
            return;
        };
        let response = match WeatherResponse::parse(response) {
            Ok(ok) => ok,
            Err(err) => {
                log::error!("{err:?}");
                return;
            }
        };
        let event = match Event::try_from(response) {
            Ok(ok) => ok,
            Err(err) => {
                log::error!("{err:?}");
                return;
            }
        };
        events.push(event);
    }

    pub(crate) fn tick(&mut self, tick: Tick) {
        if tick.is_multiple_of(120) {
            self.request_weather();
        }
    }
}
