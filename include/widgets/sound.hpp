#pragma once

#include "include/utils/subscriber.hpp"
#include <gtkmm.h>

namespace widgets {

class Sound : public Gtk::Box, public utils::Subscriber {
public:
  Sound(io::Ctx *ctx);

  void on_io_event(io::Event::Volume_Body data) override;

private:
  Gtk::Image image;
};

} // namespace widgets
