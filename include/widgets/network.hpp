#pragma once

#include "include/utils/subscriber.hpp"
#include "include/widgets/network/popover.hpp"
#include <gtkmm.h>

namespace widgets {

class Network : public Gtk::Button, public utils::Subscriber {
public:
  Network(io::UiCtx *ui_ctx);
  void on_io_event(io::Event::WifiStatus_Body data) override;
  void on_io_event(io::Event::NetworkSpeed_Body data) override;
  void on_io_event(io::Event::NetworkList_Body data) override;

private:
  Gtk::Label label;
  Gtk::Image image;

  Gtk::Label download_speed_label;
  Gtk::Image download_speed_icon;

  Gtk::Label upload_speed_label;
  Gtk::Image upload_speed_icon;

  network::Popover popover;
};

} // namespace widgets
