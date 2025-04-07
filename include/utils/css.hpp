#pragma once

#include "include/utils/subscriber.hpp"
#include <gtkmm.h>

namespace utils {

class Css : public Subscriber {
public:
  Css(io::Subscriptions *subs);
  void load();
  void on_reload_styles() override;

protected:
  std::string main_css();
  std::string theme_css();

  Glib::RefPtr<Gtk::CssProvider> provider;
};

} // namespace utils
