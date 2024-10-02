pub(crate) fn load_css() {
    let provider = gtk4::CssProvider::new();

    provider.connect_parsing_error(|_, section, error| {
        eprintln!(
            "Failed to parse CSS: {} {}",
            section.to_str(),
            error.message()
        );
    });

    let theme = std::fs::read_to_string(format!("{}/.theme.css", std::env::var("HOME").unwrap()))
        .unwrap_or_default();
    let builtin = include_str!("../../../main.css");
    let css = format!("{}\n{}", theme, builtin);

    provider.load_from_string(&css);

    let display = gtk4::gdk::Display::default().unwrap();

    #[allow(deprecated)]
    gtk4::StyleContext::add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
