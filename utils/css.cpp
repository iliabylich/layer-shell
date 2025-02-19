#include "include/utils/css.hpp"
#include <fstream>
#include <gtkmm.h>
#include <iostream>

namespace utils {

void Css::load() {
  auto full_css = theme_css() + "\n" + main_css();

  auto provider = Gtk::CssProvider::create();
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

std::string Css::main_css() {
  const char main_css_ptr[] = {
#embed "../main.css" if_empty('-')
      , 0};
  return main_css_ptr;
}

std::string Css::theme_css() {
  auto path = std::format("{}/.theme.css", getenv("HOME"));
  std::ifstream f(path);
  if (!f) {
    return "";
  }
  return std::string((std::istreambuf_iterator<char>(f)),
                     std::istreambuf_iterator<char>());
}

} // namespace utils
