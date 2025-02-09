#pragma once

#include "include/utils/subscriber.hpp"
#include <gtkmm.h>

namespace widgets {

class Tray : public Gtk::Box, public utils::Subscriber {
public:
  Tray(void *ctx);
  void on_tray_event(layer_shell_io::Event::Tray_Body data) override;

private:
  class TrayIcon {
  public:
    TrayIcon(layer_shell_io::TrayApp);
  };

  void cleanup();
  void add(layer_shell_io::TrayApp app);
};

} // namespace widgets
