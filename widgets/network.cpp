#include "include/widgets/network.hpp"
#include "include/application.hpp"
#include "include/utils/icons.hpp"
#include "include/windows/ping.hpp"

namespace widgets {

Network::Popover::Popover() : Gtk::PopoverMenu() {
  model = Gio::Menu::create();
  add_settings();
  add_ping();

  set_menu_model(model);
  set_has_arrow(false);
}

void Network::Popover::replace_networks(
    layer_shell_io::CArray<layer_shell_io::Network> networks) {
  model->remove_all();
  for (size_t i = 0; i < networks.len; i++) {
    auto network = networks.ptr[i];

    std::string iface(network.iface);
    std::string address(network.address);

    auto item = Gio::MenuItem::create(iface + ": " + address, "");
    item->set_action_and_target("network.clicked",
                                Glib::create_variant<Glib::ustring>(address));
    model->append_item(item);
  }
  add_settings();
  add_ping();
}

void Network::Popover::add_settings() {
  auto item = Gio::MenuItem::create("Settings (nmtui)", "");
  item->set_action("network.settings");
  model->append_item(item);
}

void Network::Popover::add_ping() {
  auto item = Gio::MenuItem::create("Ping", "");
  item->set_action("network.ping");
  model->append_item(item);
}

Network::Network(void *ctx) : Gtk::Button(), utils::Subscriber(ctx) {
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

  popover.set_parent(*this);

  auto action_group = Gio::SimpleActionGroup::create();
  {
    auto action = Gio::SimpleAction::create("settings");
    action->signal_activate().connect([ctx](const Glib::VariantBase &) {
      layer_shell_io::layer_shell_io_spawn_network_editor(ctx);
    });
    action_group->add_action(action);
  }
  {
    auto action = Gio::SimpleAction::create("ping");
    action->signal_activate().connect(
        [](const Glib::VariantBase &) { windows::Ping::get()->toggle(); });
    action_group->add_action(action);
  }
  {
    auto action = Gio::SimpleAction::create("clicked", Glib::VariantType("s"));
    action->signal_activate().connect([](const Glib::VariantBase &parameter) {
      auto ip =
          parameter.cast_dynamic<Glib::Variant<Glib::ustring>>(parameter).get();

      auto display = Gdk::Display::get_default();
      auto clipboard = display->get_clipboard();
      clipboard->set_text(ip);

      auto notification =
          Gio::Notification::create(std::format("Copied {}", ip.c_str()));
      get_app()->send_notification(notification);
    });
    action_group->add_action(action);
  }
  insert_action_group("network", action_group);

  signal_clicked().connect([this]() { this->popover.popup(); });
}

void Network::on_io_event(layer_shell_io::Event::WifiStatus_Body data) {
  if (data.wifi_status.tag ==
      layer_shell_io::COption<layer_shell_io::WifiStatus>::Tag::None) {
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
void Network::on_io_event(layer_shell_io::Event::NetworkSpeed_Body data) {
  download_speed_label.set_label(data.download_speed);
  upload_speed_label.set_label(data.upload_speed);
}
void Network::on_io_event(layer_shell_io::Event::NetworkList_Body data) {
  popover.replace_networks(data.list);
}

} // namespace widgets
