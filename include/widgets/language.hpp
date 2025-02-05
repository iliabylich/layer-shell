#pragma once

#include "include/utils/subscription.hpp"
#include <gtkmm.h>

namespace widgets {

class Language : public Gtk::CenterBox, public utils::Subscription<Language> {
public:
  Language();
  void activate();
  void on_io_event(const layer_shell_io::Event *event);

private:
  Gtk::Label label;
};

} // namespace widgets
