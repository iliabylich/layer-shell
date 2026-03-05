use crate::{
    liburing::IoUring,
    sansio::{Https, HttpsRequest, Satisfy, Wants},
    user_data::{ModuleId, UserData},
};
use response::LocationResponse;

mod response;

const HOST: &str = "myip.ibylich.dev";

pub(crate) struct Location {
    https: Https,
}

impl Location {
    pub(crate) const MODULE_ID: ModuleId = ModuleId::GeoLocation;

    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            https: Https::new(HttpsRequest::get(HOST, "/")),
        })
    }

    fn schedule_wanted_operation(&mut self) {
        let mut sqe = IoUring::get_sqe();

        match self.https.wants() {
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

    pub(crate) fn init(&mut self) {
        self.schedule_wanted_operation();
    }

    pub(crate) fn satisfy(&mut self, op: u8, res: i32) -> Option<(f64, f64)> {
        let satisfy = Satisfy::from(op);

        let response = match self.https.satisfy(satisfy, res) {
            Ok(Some(response)) => response,
            Ok(None) => {
                self.schedule_wanted_operation();
                return None;
            }
            Err(err) => {
                log::error!(target: "Location", "{err:?}");
                return None;
            }
        };

        match LocationResponse::parse(response) {
            Ok(response) => Some(response),
            Err(err) => {
                log::error!(target: "Location", "{err:?}");
                None
            }
        }
    }
}
