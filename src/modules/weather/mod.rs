use crate::{
    Event,
    liburing::IoUring,
    modules::weather::weather_response::WeatherResponse,
    sansio::{Https, HttpsRequest, Satisfy, Wants},
    user_data::{ModuleId, UserData},
};
pub use weather_code::WeatherCode;
pub use weather_response::{WeatherOnDay, WeatherOnHour};

mod weather_code;
mod weather_response;

const HOST: &str = "api.open-meteo.com";

pub(crate) struct Weather {
    lat: f64,
    lng: f64,
    https: Https,
}

fn path(lat: f64, lng: f64) -> String {
    let query = format!(
        "{}={}&{}={}&{}={}&{}={}&{}={}&{}={}&{}={}",
        "latitude",
        lat,
        "longitude",
        lng,
        "current",
        "temperature_2m,weather_code",
        "hourly",
        "temperature_2m,weather_code",
        "daily",
        "temperature_2m_min,temperature_2m_max,weather_code",
        "timezone",
        "Europe/Warsaw",
        "timeformat",
        "unixtime"
    );

    format!("/v1/forecast?{query}")
}

impl Weather {
    pub(crate) const MODULE_ID: ModuleId = ModuleId::Weather;

    pub(crate) fn new(lat: f64, lng: f64) -> Box<Self> {
        let mut weather = Box::new(Self {
            lat,
            lng,
            https: Https::new(HttpsRequest::get(HOST, path(lat, lng))),
        });
        weather.schedule_wanted_operation();
        weather
    }

    fn schedule_wanted_operation(&mut self) {
        let mut sqe = IoUring::get_sqe();

        match self.https.wants().unwrap() {
            Wants::Socket { domain, r#type } => {
                sqe.prep_socket(domain, r#type, 0, 0);
                sqe.set_user_data(UserData::new(Self::MODULE_ID, Satisfy::Socket));
            }
            Wants::Connect { fd, addr, addrlen } => {
                sqe.prep_connect(fd, addr, addrlen);
                sqe.set_user_data(UserData::new(Self::MODULE_ID, Satisfy::Connect));
            }
            Wants::Read { fd, buf } => {
                sqe.prep_read(fd, buf.as_mut_ptr(), buf.len());
                sqe.set_user_data(UserData::new(Self::MODULE_ID, Satisfy::Read));
            }
            Wants::Write { fd, buf } => {
                sqe.prep_write(fd, buf.as_ptr(), buf.len());
                sqe.set_user_data(UserData::new(Self::MODULE_ID, Satisfy::Write));
            }
            Wants::Close { fd } => {
                sqe.prep_close(fd);
                sqe.set_user_data(UserData::new(Self::MODULE_ID, Satisfy::Close));
            }
            Wants::Nothing => unreachable!(),
        }
    }

    fn reset(&mut self) {
        self.https = Https::new(HttpsRequest::get(HOST, path(self.lat, self.lng)));
        self.schedule_wanted_operation();
    }

    pub(crate) fn process(&mut self, op: u8, res: i32, events: &mut Vec<Event>) {
        let satisfy = Satisfy::from(op);

        let response = match self.https.satisfy(satisfy, res) {
            Ok(Some(response)) => response,
            Ok(None) => {
                self.schedule_wanted_operation();
                return;
            }
            Err(err) => {
                log::error!(target: "Location", "{err:?}");
                return;
            }
        };

        let response = match WeatherResponse::parse(response) {
            Ok(response) => response,
            Err(err) => {
                log::error!("{err:?}");
                return;
            }
        };

        let event = match Event::try_from(response) {
            Ok(event) => event,
            Err(err) => {
                log::error!("{err:?}");
                return;
            }
        };
        events.push(event);
    }

    pub(crate) fn tick(&mut self, tick: u64) {
        if tick.is_multiple_of(120) {
            self.reset();
        }
    }
}
