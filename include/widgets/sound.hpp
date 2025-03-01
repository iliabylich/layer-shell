#pragma once

#include "include/utils/subscriber.hpp"
#include <gtkmm.h>

namespace widgets {

class Sound : public Gtk::Box, public utils::Subscriber {
public:
  Sound(void *ctx);

  void on_io_event(layer_shell_io::Event::Volume_Body data) override;

private:
  Gtk::Image image;
};

} // namespace widgets
