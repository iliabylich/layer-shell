#pragma once

#include "bindings.hpp"
#include "include/utils/subscription.hpp"
#include "include/utils/widget-helper.hpp"
#include <gtkmm.h>

namespace widgets {

class Network : public Gtk::Button,
                public utils::Subscription<Network>,
                public utils::WidgetHelper<Network> {
public:
  Network();
  void activate();
  void on_io_event(const layer_shell_io::Event *event);

private:
  Gtk::Label label;
  Gtk::Image image;

  Gtk::Label download_speed_label;
  Gtk::Image download_speed_icon;

  Gtk::Label upload_speed_label;
  Gtk::Image upload_speed_icon;
};

} // namespace widgets
