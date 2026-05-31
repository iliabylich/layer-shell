use crate::modules::SessionDBus;
use anyhow::Result;
pub(crate) use control_request::ControlRequest;
use dbus::{IncomingMessage, messaging::DBusEncode};
use introspection::Introspection;
use request_name::RequestNameLayerShellControl;

mod control_request;
mod introspection;
mod request_name;

pub(crate) struct Control;

impl Control {
    pub(crate) fn init() -> Result<()> {
        let mut buf = [0; 1_024];
        let buf = RequestNameLayerShellControl::encode((), &mut buf)?;
        SessionDBus::queue().push_raw_buf(buf);
        Ok(())
    }

    pub(crate) fn handle(message: IncomingMessage<'_>) -> Option<ControlRequest> {
        match Self::try_handle(message) {
            Ok(control_request) => control_request,
            Err(err) => {
                log::error!("{err:?}");
                None
            }
        }
    }

    fn try_handle(message: IncomingMessage<'_>) -> Result<Option<ControlRequest>> {
        if Introspection::handle(message)? {
            Ok(None)
        } else {
            ControlRequest::handle(message)
        }
    }
}
