use crate::dbus::messages::{
    destination_is, introspect::IntrospectRequest as GenericIntrospectRequest, path_is,
};
use anyhow::Result;

pub(crate) struct IntrospectRequest;

impl TryFrom<&GenericIntrospectRequest<'_>> for IntrospectRequest {
    type Error = anyhow::Error;

    fn try_from(
        GenericIntrospectRequest {
            destination, path, ..
        }: &GenericIntrospectRequest,
    ) -> Result<Self> {
        destination_is!(destination, "org.me.LayerShellTmpControl");
        path_is!(path, "/");
        Ok(Self)
    }
}
