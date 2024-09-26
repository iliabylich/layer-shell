use crate::utils::{singleton, HyprlandClient, HyprlandEvent};
use anyhow::{Context, Result};

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
            let this = Self::get();
            match Self::load_initial_data().await {
                Ok(layout) => this.changed(layout),
                Err(err) => {
                    eprintln!("Failed to get hyprland language\n{}", err);
                }
            };
        });
    }

    fn changed(&self, lang: String) {
        (self.on_change)(lang)
    }

    async fn load_initial_data() -> Result<String> {
        let devices = HyprlandClient::get_devices().await;
        let main_keyboard = devices
            .keyboards
            .into_iter()
            .find(|keyboard| keyboard.main)
            .context("no keyboard is marked as 'main'")?;

        Ok(main_keyboard.active_keymap)
    }
}
