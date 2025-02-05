#pragma once

#include "include/utils/subscription.hpp"
#include <gtkmm.h>

namespace widgets {

class Workspaces : public Gtk::Box, public utils::Subscription<Workspaces> {
public:
  Workspaces();
  void activate();
  void on_io_event(const layer_shell_io::Event *event);

private:
  std::vector<Gtk::Button> buttons;
};

} // namespace widgets
