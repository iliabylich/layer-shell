pub(crate) mod introspect;
pub(crate) mod org_freedesktop_dbus;

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

macro_rules! sender_is {
    ($sender:expr, $expected:expr) => {{
        if $sender != $expected {
            anyhow::bail!("expected sender to be {:?}, got {:?}", $expected, $sender);
        }
    }};
}
pub(crate) use sender_is;

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
