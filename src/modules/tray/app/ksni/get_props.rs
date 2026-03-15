use super::{AllPropsUpdate, parse};
use crate::{
    dbus::{
        Message, OneshotResource,
        decoder::{Body, Value},
        messages::{org_freedesktop_dbus::GetAllProperties, value_is},
    },
    modules::TrayIcon,
};
use anyhow::{Context, Result, bail};

pub(crate) struct GetAllPropsOneshot;

#[derive(Debug)]
pub(crate) struct AllProps {
    pub(crate) menu: String,
    pub(crate) icon: TrayIcon,
}

impl OneshotResource for GetAllPropsOneshot {
    type Input = String;
    type Output = AllProps;

    fn make_request(&self, input: Self::Input) -> Message<'static> {
        GetAllProperties::new(input, "/StatusNotifierItem", "org.kde.StatusNotifierItem").into()
    }

    fn try_process(&self, mut body: Body<'_>) -> Result<Self::Output> {
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
