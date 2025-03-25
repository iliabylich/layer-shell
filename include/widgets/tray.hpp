#pragma once

#include "include/utils/subscriber.hpp"
#include <gtkmm.h>

namespace widgets {

class Tray : public Gtk::Box, public utils::Subscriber {
public:
  Tray(void *ctx);
  void on_io_event(io::Event::Tray_Body data) override;

private:
  class TrayIcon {
  public:
    TrayIcon(io::TrayApp);
  };

  void cleanup();
  void add(io::TrayApp app);
};

} // namespace widgets
