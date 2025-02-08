#pragma once

#include "include/utils/subscription.hpp"
#include <gtkmm.h>

namespace widgets {

class Weather : public Gtk::Button, public utils::Subscription<Weather> {
public:
  Weather();
  void activate();
  void on_io_event(const layer_shell_io::Event *event);
};

} // namespace widgets
