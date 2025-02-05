#include "include/widgets/tray.hpp"
#include "bindings.hpp"
#include "include/utils/icons.hpp"

namespace widgets {

#define ICONS_COUNT 10

Tray::Tray() : Gtk::Box() {
  set_orientation(Gtk::Orientation::HORIZONTAL);
  set_spacing(10);
  set_css_classes({"widget", "tray", "padded"});
  set_name("Tray");

  auto action_group = Gio::SimpleActionGroup::create();
  auto action = Gio::SimpleAction::create("clicked", Glib::VariantType("s"));
  action->signal_activate().connect([](const Glib::VariantBase &parameter) {
    auto uuid =
        parameter.cast_dynamic<Glib::Variant<Glib::ustring>>(parameter).get();
    layer_shell_io::layer_shell_io_trigger_tray(uuid.c_str());
  });
  action_group->add_action(action);
  insert_action_group("tray", action_group);
}

void Tray::activate() { subscribe_to_io_events(); }

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
  gesture->set_button(3 /* right click */);
  icon.add_controller(gesture);

  gesture->signal_pressed().connect([popover_menu](int, double x, double y) {
    Gdk::Rectangle rect(x, y, 1, 1);
    popover_menu->set_pointing_to(rect);
    popover_menu->popup();
  });

  append(icon);
}

void Tray::on_io_event(const layer_shell_io::Event *event) {
  if (event->tag == layer_shell_io::Event::Tag::Tray) {
    cleanup();

    auto apps = event->tray.list;
    for (size_t i = 0; i < ICONS_COUNT; i++) {
      if (i < apps.len) {
        add(apps.ptr[i]);
      }
    }
  }
}

} // namespace widgets
