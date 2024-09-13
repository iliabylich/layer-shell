pub(crate) fn load_css() {
    let provider = gtk4::CssProvider::new();

    provider.connect_parsing_error(|_, section, error| {
        eprintln!(
            "Failed to parse CSS: {} {}",
            section.to_str(),
            error.message()
        );
    });

    let path = format!(
        "{}/.config/layer-shell/css/style.css",
        std::env::var("HOME").unwrap()
    );
    provider.load_from_path(path);

    let display = gtk4::gdk::Display::default().unwrap();

    #[allow(deprecated)]
    gtk4::StyleContext::add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
