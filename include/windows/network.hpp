#pragma once

#include "include/utils/subscription.hpp"
#include "include/utils/window-helper.hpp"
#include <gtkmm.h>

namespace windows {

class Network : public Gtk::Window,
                public utils::Subscription<Network>,
                public utils::WindowHelper<Network> {
public:
  static int WIDTH;
  Network();
  void activate(const Glib::RefPtr<Gtk::Application> &app);
  void on_io_event(const layer_shell_io::Event *event);

private:
  class Row : public Gtk::CenterBox {
  public:
    Row() = default;
    Row(const char *text, const char *icon_name);

  protected:
    Gtk::Label label;
    Gtk::Image image;
  };

  class NetworkRow : public Row {
  public:
    NetworkRow();
    void activate();
    void update(const std::string &ip, const std::string &iface);

  private:
    std::string ip;
    std::string iface;
  };

  class SettingsRow : public Row {
  public:
    SettingsRow();
    void activate();
  };

  class ExitRow : public Row {
  public:
    ExitRow();
    void activate();
  };

  std::vector<NetworkRow> rows;
  SettingsRow settings_row;
  ExitRow exit_row;
};

} // namespace windows
