#include "include/widgets/network.hpp"
#include "bindings.hpp"
#include "include/utils/icons.hpp"
#include "include/windows/network.hpp"
#include "include/windows/top-bar.hpp"

namespace widgets {

Network::Network() : Gtk::Button() {
  label.set_label("--");

  image.set(utils::Icons::wifi_icon());

  download_speed_label.set_label("??");
  download_speed_label.set_css_classes({"network-speed-label"});
  download_speed_icon.set(utils::Icons::download_speed_icon());

  upload_speed_label.set_label("??");
  upload_speed_label.set_css_classes({"network-speed-label"});
  upload_speed_icon.set(utils::Icons::upload_speed_icon());

  Gtk::Box network_wrapper(Gtk::Orientation::HORIZONTAL, 0);
  network_wrapper.append(label);
  network_wrapper.append(image);

  Gtk::Separator sep(Gtk::Orientation::HORIZONTAL);
  network_wrapper.append(sep);

  network_wrapper.append(download_speed_label);
  network_wrapper.append(download_speed_icon);
  network_wrapper.append(upload_speed_label);
  network_wrapper.append(upload_speed_icon);

  set_css_classes({"widget", "network", "padded", "clickable"});
  set_name("Network");
  set_cursor("pointer");
  set_child(network_wrapper);
}

void Network::activate() {
  compute_bounds(*this);
  signal_clicked().connect([this]() {
    auto bottom_right = this->bottom_right_point(*windows::TopBar::instance());
    windows::Network::move(bottom_right.get_x() - windows::Network::WIDTH,
                           bottom_right.get_y());
    windows::Network::toggle();
  });

  subscribe_to_io_events();
}

void Network::on_io_event(const layer_shell_io::Event *event) {
  if (event->tag == layer_shell_io::Event::Tag::WifiStatus) {
    if (event->wifi_status.wifi_status.tag ==
        layer_shell_io::COption<layer_shell_io::WifiStatus>::Tag::None) {
      image.hide();
      label.set_label("Not connected");
    } else {
      image.show();
      char buffer[100];
      sprintf(buffer, "%s (%d)%% ", event->wifi_status.wifi_status.some._0.ssid,
              event->wifi_status.wifi_status.some._0.strength);
      label.set_label(buffer);
    }
  } else if (event->tag == layer_shell_io::Event::Tag::NetworkSpeed) {
    download_speed_label.set_label(event->network_speed.download_speed);
    upload_speed_label.set_label(event->network_speed.upload_speed);
  }
}

} // namespace widgets
