#include "include/utils/css.hpp"
#include "main.scss.xxd"
#include <fstream>
#include <gtkmm.h>
#include <iostream>

namespace utils {

Css::Css(io::UiCtx *ui_ctx) : Subscriber(ui_ctx) {
  provider = std::shared_ptr<Gtk::CssProvider>(nullptr);
}

void Css::load() {
  auto full_css = theme_css() + "\n" + main_css();

  provider = Gtk::CssProvider::create();
  provider->signal_parsing_error().connect(
      [](const Glib::RefPtr<const Gtk::CssSection> &section,
         const Glib::Error &error) {
        std::cerr << "Failed to parse CSS: " << section->to_string() << " "
                  << error.what() << "\n";
      });
  provider->load_from_string(full_css);

  auto display = Gdk::Display::get_default();
  Gtk::StyleContext::add_provider_for_display(
      display, provider, GTK_STYLE_PROVIDER_PRIORITY_APPLICATION);
  std::cout << "Finished loading CSS...\n";
}

void Css::on_reload_styles() {
  std::cout << "Reloading styles...\n";
  auto display = Gdk::Display::get_default();
  Gtk::StyleContext::remove_provider_for_display(display, provider);
  load();
}

std::string Css::main_css() {
  return std::string((char *)layer_shell_p_main_scss,
                     layer_shell_p_main_scss_len);
}

std::string Css::theme_css() {
  auto path = std::format("{}/.config/layer-shell/theme.css", getenv("HOME"));
  std::ifstream f(path);
  if (!f) {
    return "";
  }
  return std::string((std::istreambuf_iterator<char>(f)),
                     std::istreambuf_iterator<char>());
}

} // namespace utils
