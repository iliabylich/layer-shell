#pragma once

#include "include/utils/subscriber.hpp"
#include <gtkmm.h>

namespace widgets {

class Workspaces : public Gtk::Box, public utils::Subscriber {
public:
  Workspaces(void *ctx);
  void on_io_event(io::Event::Workspaces_Body data) override;

private:
  std::vector<Gtk::Button> buttons;
};

} // namespace widgets
