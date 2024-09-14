use crate::{models::singleton, utils::exec_async};

pub(crate) struct Logout {
    idx: usize,
    max: usize,
    on_change: Box<dyn Fn(usize)>,
}

singleton!(Logout);

impl Logout {
    pub(crate) fn spawn<F>(max: usize, on_change: F)
    where
        F: Fn(usize) + 'static,
    {
        Self::set(Self {
            idx: 0,
            max,
            on_change: Box::new(on_change),
        })
    }

    pub(crate) fn reset() {
        Self::get().idx = 0;
        Self::changed();
    }

    pub(crate) fn lock() {
        gtk4::glib::spawn_future_local(async {
            exec_async(&["hyprlock"]).await;
        });
    }
    pub(crate) fn reboot() {
        gtk4::glib::spawn_future_local(async {
            exec_async(&["systemctl", "reboot"]).await;
        });
    }
    pub(crate) fn shutdown() {
        gtk4::glib::spawn_future_local(async {
            exec_async(&["systemctl", "poweroff"]).await;
        });
    }
    pub(crate) fn logout() {
        gtk4::glib::spawn_future_local(async {
            exec_async(&["hyprctl", "dispatch", "exit"]).await;
        });
    }

    pub(crate) fn left() {
        if Self::get().idx == 0 {
            return;
        }
        Self::get().idx = std::cmp::max(0, Self::get().idx - 1);
        Self::changed();
    }
    pub(crate) fn right() {
        Self::get().idx = std::cmp::min(Self::get().max - 1, Self::get().idx + 1);
        Self::changed();
    }

    fn changed() {
        let instance = Self::get();
        (instance.on_change)(instance.idx);
    }
}
