use crate::utils::{exec_async, singleton};

pub(crate) struct Logout {
    idx: usize,
    max: usize,
    on_change: Box<dyn Fn(usize)>,
}
singleton!(Logout);

impl Logout {
    pub(crate) fn subscribe<F>(max: usize, on_change: F)
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
        this().idx = 0;
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
        if this().idx == 0 {
            return;
        }
        this().idx = std::cmp::max(0, this().idx - 1);
        Self::changed();
    }
    pub(crate) fn right() {
        this().idx = std::cmp::min(this().max - 1, this().idx + 1);
        Self::changed();
    }

    fn changed() {
        let instance = this();
        (instance.on_change)(instance.idx);
    }
}
