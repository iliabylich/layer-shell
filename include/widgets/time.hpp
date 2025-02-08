#pragma once

#include "include/utils/subscription.hpp"
#include <gtkmm.h>

namespace widgets {

class Time : public Gtk::Label, public utils::Subscription<Time> {
public:
  Time();
  void activate(void *subscriptions);
  void on_io_event(const layer_shell_io::Event *event);
};

} // namespace widgets
