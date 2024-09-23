use crate::utils::{singleton, HyprlandClient, HyprlandEvent};

pub(crate) struct HyprlandLanguage {
    on_change: Box<dyn Fn(String)>,
}
singleton!(HyprlandLanguage);

impl HyprlandLanguage {
    pub(crate) fn subscribe<F>(f: F)
    where
        F: Fn(String) + 'static,
    {
        Self::set(Self {
            on_change: Box::new(f),
        });

        HyprlandClient::subscribe(|event| {
            if let HyprlandEvent::LanguageChanged(new_lang) = event {
                Self::get().changed(new_lang);
            }
        });

        gtk4::glib::spawn_future_local(async {
            Self::get().load_initial_data().await;
        });
    }

    fn changed(&self, lang: String) {
        (self.on_change)(lang)
    }

    async fn load_initial_data(&self) {
        let devices = HyprlandClient::get_devices().await;
        let layout = devices
            .keyboards
            .into_iter()
            .find(|keyboard| keyboard.main)
            .unwrap()
            .active_keymap;
        self.changed(layout);
    }
}
