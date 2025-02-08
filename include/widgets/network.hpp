#pragma once

#include "bindings.hpp"
#include "glibmm/refptr.h"
#include "gtkmm/popovermenu.h"
#include "include/utils/subscription.hpp"
#include <gtkmm.h>

namespace widgets {

class Network : public Gtk::Button, public utils::Subscription<Network> {
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

  class Popover : public Gtk::PopoverMenu {
  public:
    Popover();
    void
    replace_networks(layer_shell_io::CArray<layer_shell_io::Network> networks);

  private:
    Glib::RefPtr<Gio::Menu> model;
    void add_settings();
  };

  Popover popover;
};

} // namespace widgets
