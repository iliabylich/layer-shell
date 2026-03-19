use super::{AllPropsUpdate, parse};
use crate::{
    dbus::{
        OneshotResource, OutgoingMessage,
        decoder::{Body, Value},
        messages::{org_freedesktop_dbus::GetAllProperties, value_is},
    },
    ffi::ShortString,
    modules::TrayIcon,
};
use anyhow::{Context, Result, bail};

pub(crate) struct GetAllPropsOneshot;

#[derive(Debug)]
pub(crate) struct AllProps {
    pub(crate) menu: ShortString,
    pub(crate) icon: TrayIcon,
}

impl OneshotResource for GetAllPropsOneshot {
    type Input = ShortString;
    type Output = AllProps;

    fn request(&self, destination: ShortString) -> impl Into<OutgoingMessage> {
        GetAllProperties::new(
            destination,
            ShortString::new_const("/StatusNotifierItem"),
            ShortString::new_const("org.kde.StatusNotifierItem"),
        )
    }

    fn try_recv(&self, mut body: Body<'_>) -> Result<Self::Output> {
        let array = body.try_next()?.context("no array")?;
        value_is!(array, Value::Array(array));
        match parse(array)? {
            AllPropsUpdate {
                menu: Some(menu),
                icon: Some(icon),
            } => Ok(AllProps { menu, icon }),

            other => {
                log::error!(
                    "initial GetAllProps request for tray app failed, some data is missing: {other:?}"
                );
                bail!("DBus internal error")
            }
        }
    }
}
