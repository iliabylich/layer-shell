#include "include/widgets/network.hpp"
#include "include/utils/icons.hpp"

namespace widgets {

Network::Network(void *ctx)
    : Gtk::Button(), utils::Subscriber(ctx), popover(ctx) {
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

  Gtk::Separator sep(Gtk::Orientation::VERTICAL);
  network_wrapper.append(sep);

  network_wrapper.append(download_speed_label);
  network_wrapper.append(download_speed_icon);
  network_wrapper.append(upload_speed_label);
  network_wrapper.append(upload_speed_icon);

  set_css_classes({"widget", "network", "padded", "clickable"});
  set_name("Network");
  set_cursor("pointer");
  set_child(network_wrapper);

  popover.set_parent(*this);

  signal_clicked().connect([this]() { this->popover.popup(); });
}

void Network::on_io_event(io::Event::WifiStatus_Body data) {
  if (data.wifi_status.tag == io::COption<io::WifiStatus>::Tag::None) {
    image.hide();
    label.set_label("Not connected");
  } else {
    image.show();
    char buffer[100];
    sprintf(buffer, "%s (%d)%% ", data.wifi_status.some._0.ssid,
            data.wifi_status.some._0.strength);
    label.set_label(buffer);
  }
}
void Network::on_io_event(io::Event::NetworkSpeed_Body data) {
  download_speed_label.set_label(data.download_speed);
  upload_speed_label.set_label(data.upload_speed);
}
void Network::on_io_event(io::Event::NetworkList_Body data) {
  popover.update(data.list);
}

} // namespace widgets
