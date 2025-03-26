#pragma once

#include "src/bindings.hpp"
#include <gtkmm.h>

namespace widgets {
namespace network {

class Popover : public Gtk::PopoverMenu {
public:
  Popover(io::Ctx *ctx);
  void update(io::CArray<io::Network> networks);

private:
  Glib::RefPtr<Gio::Menu> model;
  io::Ctx *ctx;

  void add_settings_row();
  void add_ping_row();

  void on_settings_row_clicked();
  void on_ping_row_clicked();
  void on_network_row_clicked(const Glib::VariantBase &parameter);
};

} // namespace network
} // namespace widgets
