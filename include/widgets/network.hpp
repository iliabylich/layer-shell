#pragma once

#include "include/utils/subscriber.hpp"
#include <gtkmm.h>

namespace widgets {

class Network : public Gtk::Button, public utils::Subscriber {
public:
  Network(void *ctx);
  void
  on_wifi_status_event(layer_shell_io::Event::WifiStatus_Body data) override;
  void on_network_speed_event(
      layer_shell_io::Event::NetworkSpeed_Body data) override;
  void
  on_network_list_event(layer_shell_io::Event::NetworkList_Body data) override;

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
