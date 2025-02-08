#pragma once

#include "include/utils/subscription.hpp"
#include <gtkmm.h>

namespace widgets {

class Language : public Gtk::Label, public utils::Subscription<Language> {
public:
  Language();
  void activate(void *subscriptions);
  void on_io_event(const layer_shell_io::Event *event);
};

} // namespace widgets
