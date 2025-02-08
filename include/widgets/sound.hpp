#pragma once

#include "include/utils/subscription.hpp"
#include <gtkmm.h>

namespace widgets {

class Sound : public Gtk::Box, public utils::Subscription<Sound> {
public:
  Sound();
  void activate(void *subscriptions);
  void on_io_event(const layer_shell_io::Event *event);

private:
  Gtk::Image image;
  Gtk::Scale scale;
};

} // namespace widgets
