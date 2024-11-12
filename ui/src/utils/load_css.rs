use anyhow::{Context, Result};

pub(crate) fn load_css() -> Result<()> {
    let provider = gtk4::CssProvider::new();

    provider.connect_parsing_error(|_, section, error| {
        eprintln!(
            "Failed to parse CSS: {} {}",
            section.to_str(),
            error.message()
        );
    });

    let home = std::env::var("HOME").context("failed to get HOME variable")?;

    let theme_filepath = format!("{}/.theme.css", home);
    let theme = std::fs::read_to_string(theme_filepath).unwrap_or_default();
    let builtin = include_str!("../../../main.css");
    let css = format!("{}\n{}", theme, builtin);

    provider.load_from_string(&css);

    let display = gtk4::gdk::Display::default().context("failed to get default Gdk display")?;
    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    Ok(())
}
