#pragma once

#include "bindings.hpp"
#include "include/utils/subscription.hpp"
#include <gtkmm.h>

namespace widgets {

class Tray : public Gtk::Box, public utils::Subscription<Tray> {
public:
  Tray();
  void activate(void *subscriptions);
  void on_io_event(const layer_shell_io::Event *event);

private:
  class TrayIcon {
  public:
    TrayIcon(layer_shell_io::TrayApp);
  };

  void cleanup();
  void add(layer_shell_io::TrayApp app);
};

} // namespace widgets
