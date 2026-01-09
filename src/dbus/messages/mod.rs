mod org_freedesktop_dbus;
pub(crate) use org_freedesktop_dbus::{
    AddMatch, Hello, NameAcquired, PropertiesChanged, RequestName,
};

mod introspect;
pub(crate) use introspect::{IntrospectRequest, IntrospectResponse};

mod pipewire;
pub(crate) use pipewire::{MuteChanged, VolumeChanged};

macro_rules! message_is {
    ($message:expr, $pat:pat) => {
        let $pat = $message else {
            anyhow::bail!(
                "expected Message::{}, got {:?}",
                stringify!($expected),
                $message
            );
        };
    };
}
pub(crate) use message_is;

macro_rules! interface_is {
    ($interface:expr, $expected:expr) => {{
        if $interface != $expected {
            anyhow::bail!(
                "expected interface to be {:?}, got {:?}",
                $expected,
                $interface
            );
        }
    }};
}
pub(crate) use interface_is;

macro_rules! destination_is {
    ($destination:expr, $expected:expr) => {{
        if $destination != $expected {
            anyhow::bail!(
                "expected destination to be {:?}, got {:?}",
                $expected,
                $destination
            );
        }
    }};
}
pub(crate) use destination_is;

macro_rules! path_is {
    ($path:expr, $expected:expr) => {{
        if $path != $expected {
            anyhow::bail!("expected path to be {:?}, got {:?}", $expected, $path);
        }
    }};
}
pub(crate) use path_is;

macro_rules! member_is {
    ($member:expr, $expected:expr) => {{
        if $member != $expected {
            anyhow::bail!("expected member to be {:?}, got {:?}", $expected, $member);
        }
    }};
}
pub(crate) use member_is;

pub(crate) fn as_array<T, const N: usize>(slice: &[T]) -> Option<&[T; N]> {
    slice.try_into().ok()
}

macro_rules! body_is {
    ($body:expr, $expected:pat) => {
        let Some($expected) = $crate::dbus::messages::as_array($body) else {
            anyhow::bail!("body format mismatch: {:?}", $body);
        };
    };
}
pub(crate) use body_is;

macro_rules! value_is {
    ($value:expr, $pat:pat) => {
        let $pat = $value else {
            anyhow::bail!("value format mismatch: {:?}", $value);
        };
    };
}
pub(crate) use value_is;

macro_rules! type_is {
    ($type:expr, $pat:pat) => {
        let $pat = $type else {
            anyhow::bail!("type mismatch: {:?}", $type);
        };
    };
}
pub(crate) use type_is;

macro_rules! define_sum_message {
    ($enum_name:ident, $($variant:ident$(<$lt:lifetime>)?),+ $(,)?) => {
        #[derive(Debug)]
        pub(crate) enum $enum_name<'a> {
            $(
                $variant($variant$(<$lt>)?),
            )+
        }

        impl<'a> TryFrom<&'a $crate::dbus::Message> for $enum_name<'a> {
            type Error = anyhow::Error;

            fn try_from(message: &'a $crate::dbus::Message) -> anyhow::Result<Self> {
                $(
                    if let Ok(mapped) = $variant::try_from(message) {
                        return Ok(Self::$variant(mapped));
                    }
                )+

                anyhow::bail!("{message:?} doesn't match any registered type")
            }
        }
    };
}

define_sum_message!(
    KnownDBusMessage,
    NameAcquired<'a>,
    IntrospectRequest<'a>,
    VolumeChanged,
    MuteChanged,
    PropertiesChanged<'a>
);
