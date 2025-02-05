#pragma once

#include "include/utils/subscription.hpp"
#include <gtkmm.h>

namespace widgets {

class Time : public Gtk::CenterBox, public utils::Subscription<Time> {
public:
  Time();
  void activate();
  void on_io_event(const layer_shell_io::Event *event);

private:
  Gtk::Label label;
};

} // namespace widgets
