#include "include/widgets/tray.hpp"
#include "include/utils/icons.hpp"

namespace widgets {

size_t max_icons_count = 10;

Tray::Tray(void *ctx) : Gtk::Box(), utils::Subscriber(ctx) {
  set_orientation(Gtk::Orientation::HORIZONTAL);
  set_spacing(10);
  set_css_classes({"widget", "tray", "padded"});
  set_name("Tray");
}

void Tray::cleanup() {
  for (auto child : this->get_children()) {
    this->remove(*child);
  }
}

Glib::RefPtr<Gio::Menu>
new_menu_for_tray_item(layer_shell_io::TrayItem data,
                       Glib::RefPtr<Gio::SimpleActionGroup> &action_group,
                       void *ctx) {
  auto menu = Gio::Menu::create();

  for (size_t i = 0; i < data.children.len; i++) {
    auto child = data.children.ptr[i];
    if (!child.visible) {
      continue;
    }

    auto menu_item = Gio::MenuItem::create(child.label, "");

    std::string children_display(child.children_display);
    std::string uuid(child.uuid);
    std::string toggle_type(child.toggle_type);
    std::string action_name = std::format("{}", i);

    auto cb = [ctx, uuid](const Glib::VariantBase &) {
      layer_shell_io::layer_shell_io_trigger_tray(uuid.c_str(), ctx);
    };

    if (children_display == "submenu") {
      // nested menu
      auto submenu = new_menu_for_tray_item(child, action_group, ctx);
      menu_item->set_submenu(submenu);
    } else {
      // element
      if (child.enabled) {
        if (toggle_type == "checkmark") {
          // checkbox
          auto action = Gio::SimpleAction::create(
              action_name, Glib::create_variant<bool>(child.toggle_state == 1));

          action->signal_activate().connect(cb);
          action_group->add_action(action);

          menu_item->set_action(std::string("tray.") + action_name);
        } else if (toggle_type == "radio") {
          // radio
          auto action = Gio::SimpleAction::create(
              action_name, Glib::VariantType("b"),
              Glib::create_variant<bool>(child.toggle_state == 1));

          action->signal_activate().connect(cb);
          action_group->add_action(action);

          menu_item->set_action_and_target(std::string("tray.") + action_name,
                                           Glib::create_variant<bool>(true));
        } else {
          auto action = Gio::SimpleAction::create(action_name);
          action->signal_activate().connect(cb);
          action_group->add_action(action);

          menu_item->set_action(std::string("tray.") + action_name);
        }
      } else {
        menu_item->set_action("tray.noop");
      }
    }

    menu->append_item(menu_item);
  }

  return menu;
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

  auto action_group = Gio::SimpleActionGroup::create();
  auto menu = new_menu_for_tray_item(app.root_item, action_group, ctx);

  auto popover_menu = new Gtk::PopoverMenu(menu);
  popover_menu->set_has_arrow(false);
  popover_menu->set_parent(icon);

  auto gesture = Gtk::GestureClick::create();
  gesture->signal_pressed().connect(
      [popover_menu](int, double, double) { popover_menu->popup(); });
  icon.add_controller(gesture);

  append(icon);

  icon.insert_action_group("tray", action_group);
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
