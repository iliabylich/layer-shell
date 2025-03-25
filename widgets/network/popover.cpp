#include "include/widgets/network/popover.hpp"
#include "include/application.hpp"
#include "include/windows/ping.hpp"

namespace widgets {
namespace network {

Popover::Popover(void *ctx) : Gtk::PopoverMenu(), ctx(ctx) {
  model = Gio::Menu::create();
  add_settings_row();
  add_ping_row();

  set_menu_model(model);
  set_has_arrow(false);

  auto action_group = Gio::SimpleActionGroup::create();
  {
    auto action = Gio::SimpleAction::create("settings");
    action->signal_activate().connect(
        [this](const Glib::VariantBase &) { on_settings_row_clicked(); });
    action_group->add_action(action);
  }
  {
    auto action = Gio::SimpleAction::create("ping");
    action->signal_activate().connect(
        [this](const Glib::VariantBase &) { on_ping_row_clicked(); });
    action_group->add_action(action);
  }
  {
    auto action = Gio::SimpleAction::create("clicked", Glib::VariantType("s"));
    action->signal_activate().connect(
        [this](const Glib::VariantBase &parameter) {
          on_network_row_clicked(parameter);
        });
    action_group->add_action(action);
  }
  insert_action_group("network", action_group);
}

void Popover::update(io::CArray<io::Network> networks) {
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
  add_settings_row();
  add_ping_row();
}

void Popover::add_settings_row() {
  auto item = Gio::MenuItem::create("Settings (nmtui)", "");
  item->set_action("network.settings");
  model->append_item(item);
}

void Popover::add_ping_row() {
  auto item = Gio::MenuItem::create("Ping", "");
  item->set_action("network.ping");
  model->append_item(item);
}

void Popover::on_settings_row_clicked() { io::io_spawn_network_editor(ctx); }
void Popover::on_ping_row_clicked() { windows::Ping::get()->toggle(); }
void Popover::on_network_row_clicked(const Glib::VariantBase &parameter) {
  auto ip =
      parameter.cast_dynamic<Glib::Variant<Glib::ustring>>(parameter).get();

  auto display = Gdk::Display::get_default();
  auto clipboard = display->get_clipboard();
  clipboard->set_text(ip);

  auto notification =
      Gio::Notification::create(std::format("Copied {}", ip.c_str()));
  get_app()->send_notification(notification);
}

} // namespace network
} // namespace widgets
