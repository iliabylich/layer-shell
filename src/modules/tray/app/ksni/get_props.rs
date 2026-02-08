use super::{AllPropsUpdate, parse};
use crate::{
    dbus::{
        Message, OneshotResource,
        messages::{body_is, org_freedesktop_dbus::GetAllProperties, type_is},
        types::{CompleteType, Value},
    },
    modules::TrayIcon,
};
use anyhow::{Result, bail};

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

    fn try_process(&self, body: &[Value]) -> Result<Self::Output> {
        body_is!(body, [Value::Array(item_t, array)]);
        type_is!(item_t, CompleteType::DictEntry(key_t, value_t));
        type_is!(&**key_t, CompleteType::String);
        type_is!(&**value_t, CompleteType::Variant);

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
