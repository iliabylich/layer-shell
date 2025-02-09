#pragma once

#include "include/utils/subscriber.hpp"
#include <gtkmm.h>

namespace widgets {

class Time : public Gtk::Label, public utils::Subscriber {
public:
  Time(void *ctx);
  void on_io_event(layer_shell_io::Event::Time_Body data) override;
};

} // namespace widgets
