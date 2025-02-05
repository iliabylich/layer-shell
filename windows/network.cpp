#include "include/windows/network.hpp"
#include "bindings.hpp"
#include "gtk4-layer-shell.h"

namespace windows {

Network::Row::Row(const char *text, const char *icon_name) : Gtk::CenterBox() {
  set_css_classes({"widget-network-row"});
  set_orientation(Gtk::Orientation::HORIZONTAL);
  set_halign(Gtk::Align::FILL);

  label.set_label(text);
  label.set_justify(Gtk::Justification::LEFT);
  label.set_xalign(0.0);
  set_start_widget(label);

  image.set_from_icon_name(icon_name);
  image.set_icon_size(Gtk::IconSize::LARGE);
  image.set_pixel_size(30);
  set_end_widget(image);
}

// --

Network::NetworkRow::NetworkRow() : Network::Row("--", "edit-copy") {}

void Network::NetworkRow::activate() {
  auto ctrl = Gtk::GestureClick::create();
  ctrl->signal_pressed().connect([this](int, double, double) {
    auto display = Gdk::Display::get_default();
    auto clipboard = display->get_clipboard();
    clipboard->set_text(this->ip);

    this->label.set_label("Copied!");
    Glib::signal_timeout().connect_seconds(
        [this]() {
          this->label.set_text(this->iface + ": " + this->ip);
          return false;
        },
        1);
  });
  add_controller(ctrl);
}

void Network::NetworkRow::update(const std::string &new_ip,
                                 const std::string &new_iface) {
  ip = new_ip;
  iface = new_iface;
  auto text = iface + ": " + ip;
  label.set_label(text);
  label.set_tooltip_text(ip);
}

// --

Network::SettingsRow::SettingsRow()
    : Network::Row("Settings (nmtui)", "preferences-system-network") {}

void Network::SettingsRow::activate() {
  auto ctrl = Gtk::GestureClick::create();
  ctrl->signal_pressed().connect(

      [](int, double, double) {
        windows::Network::toggle();
        layer_shell_io::layer_shell_io_spawn_network_editor();
      });
  add_controller(ctrl);
}

// --

Network::ExitRow::ExitRow() : Network::Row("Close", "window-close") {}

void Network::ExitRow::activate() {
  auto ctrl = Gtk::GestureClick::create();
  ctrl->signal_pressed().connect(
      [](int, double, double) { Network::toggle(); });
  add_controller(ctrl);
}

// --

int Network::WIDTH = 700;

Network::Network() : Gtk::Window() {
  set_name("NetworksWindow");
  property_width_request().set_value(WIDTH);

  Gtk::Box layout(Gtk::Orientation::VERTICAL, 0);
  layout.set_css_classes({"widget-network-row-list"});
  set_child(layout);

  for (size_t i = 0; i < 5; i++) {
    NetworkRow row;
    layout.append(row);
    rows.push_back(std::move(row));
  }

  layout.append(settings_row);
  layout.append(exit_row);
}

void Network::activate(const Glib::RefPtr<Gtk::Application> &app) {
  set_application(app);
  toggle_on_escape();

  auto win = gobj();
  gtk_layer_init_for_window(win);
  gtk_layer_set_layer(win, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_anchor(win, GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_anchor(win, GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_namespace(win, "LayerShell/Networks");
  gtk_layer_set_keyboard_mode(win, GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  for (auto &row : rows) {
    row.activate();
  }
  settings_row.activate();
  exit_row.activate();

  subscribe_to_io_events();
}

void Network::on_io_event(const layer_shell_io::Event *event) {
  if (event->tag == layer_shell_io::Event::Tag::NetworkList) {
    auto networks = event->network_list.list;
    for (size_t i = 0; i < 5; i++) {
      auto &row = rows.at(i);
      if (i < networks.len) {
        auto network = networks.ptr[i];
        row.update(network.address, network.iface);
        row.show();
      } else {
        row.hide();
      }
    }
  }
}

} // namespace windows
