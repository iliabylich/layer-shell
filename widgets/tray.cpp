#include "include/widgets/tray.hpp"
#include "include/utils/icons.hpp"

namespace widgets {

size_t max_icons_count = 10;

Tray::Tray(void *ctx) : Gtk::Box(), utils::Subscriber(ctx) {
  set_orientation(Gtk::Orientation::HORIZONTAL);
  set_spacing(10);
  set_css_classes({"widget", "tray", "padded"});
  set_name("Tray");

  auto action_group = Gio::SimpleActionGroup::create();
  auto action = Gio::SimpleAction::create("clicked", Glib::VariantType("s"));
  action->signal_activate().connect([ctx](const Glib::VariantBase &parameter) {
    auto uuid =
        parameter.cast_dynamic<Glib::Variant<Glib::ustring>>(parameter).get();
    layer_shell_io::layer_shell_io_trigger_tray(uuid.c_str(), ctx);
  });
  action_group->add_action(action);
  insert_action_group("tray", action_group);
}

void Tray::cleanup() {
  for (auto child : this->get_children()) {
    this->remove(*child);
  }
}

void Tray::add(layer_shell_io::TrayApp app) {
  Gtk::Image icon;

  switch (app.icon.tag) {
  case layer_shell_io::TrayIcon::Tag::Path: {
    icon.set_from_resource(app.icon.path.path);
    break;
  }
  case layer_shell_io::TrayIcon::Tag::Name: {
    icon.set_from_icon_name(app.icon.name.name);
    break;
  }
  case layer_shell_io::TrayIcon::Tag::PixmapVariant: {
    GBytes *bytes = g_bytes_new(app.icon.pixmap_variant.bytes.ptr,
                                app.icon.pixmap_variant.bytes.len);
    auto pixbuf = Glib::wrap(gdk_pixbuf_new_from_bytes(
        bytes, GDK_COLORSPACE_RGB, TRUE, 8, app.icon.pixmap_variant.w,
        app.icon.pixmap_variant.h, 4 * app.icon.pixmap_variant.w));
    auto texture = Gdk::Texture::create_for_pixbuf(pixbuf);
    icon.set(texture);
    break;
  }
  case layer_shell_io::TrayIcon::Tag::None: {
    icon.set(utils::Icons::question_mark_icon());
    break;
  }
  }

  auto menu = Gio::Menu::create();

  for (size_t i = 0; i < app.items.len; i++) {
    auto item = Gio::MenuItem::create(app.items.ptr[i].label, "");
    const char *action = "tray.clicked";
    if (app.items.ptr[i].disabled) {
      action = "tray.noop";
    }
    item->set_action_and_target(
        action, Glib::create_variant<Glib::ustring>(app.items.ptr[i].uuid));
    menu->append_item(item);
  }

  auto popover_menu = new Gtk::PopoverMenu(menu);
  popover_menu->set_has_arrow(false);
  popover_menu->set_parent(icon);

  auto gesture = Gtk::GestureClick::create();
  gesture->signal_pressed().connect(
      [popover_menu](int, double, double) { popover_menu->popup(); });
  icon.add_controller(gesture);

  append(icon);
}

void Tray::on_io_event(layer_shell_io::Event::Tray_Body data) {
  cleanup();

  auto apps = data.list;
  for (size_t i = 0; i < max_icons_count; i++) {
    if (i < apps.len) {
      add(apps.ptr[i]);
    }
  }
}

} // namespace widgets
