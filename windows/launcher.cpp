#include "include/windows/launcher.hpp"
#include <gtk4-layer-shell.h>

namespace windows {

Launcher::Row::Row() : Gtk::Box() {
  set_orientation(Gtk::Orientation::HORIZONTAL);
  set_spacing(0);
  set_css_classes({"widget-launcher-row"});

  image.set_icon_size(Gtk::IconSize::LARGE);

  label.set_label("...");
  label.set_xalign(0.0);
  label.set_valign(Gtk::Align::CENTER);
  label.set_ellipsize(Pango::EllipsizeMode::END);

  append(image);
  append(label);
}

void Launcher::Row::update(layer_shell_io::App app) {
  show();
  if (app.selected) {
    add_css_class("active");
  } else {
    remove_css_class("active");
  }

  if (app.icon.tag == layer_shell_io::AppIcon::Tag::IconName) {
    image.set_from_icon_name(app.icon.icon_name._0);
  } else {
    image.set_from_resource(app.icon.icon_path._0);
  }
  label.set_label(app.name);
}

// ----

Launcher::Launcher(const Glib::RefPtr<Gtk::Application> &app, void *ctx)
    : utils::Subscriber(ctx) {
  set_name("LauncherWindow");
  property_width_request().set_value(700);
  set_application(app);

  Gtk::Box layout(Gtk::Orientation::VERTICAL, 0);
  layout.set_css_classes({"widget-launcher-wrapper"});
  set_child(layout);

  input.set_css_classes({"widget-launcher-search-box"});
  input.set_hexpand(true);
  layout.append(input);

  Gtk::ScrolledWindow scroll;
  scroll.set_css_classes({"widget-launcher-scroll-list"});
  scroll.set_can_focus(false);
  layout.append(scroll);

  Gtk::Box content(Gtk::Orientation::VERTICAL, 0);
  scroll.set_child(content);

  for (size_t i = 0; i < 5; i++) {
    Row row;
    content.append(row);
    rows.push_back(std::move(row));
  }
}

void Launcher::toggle_and_reset() {
  Glib::signal_timeout().connect(
      [this]() {
        // Toggling immediately breaks something in gtkmm, so we
        // schedule it
        if (is_visible()) {
          hide();
        } else {
          layer_shell_io::layer_shell_io_app_list_reset(ctx);
          input.set_text("");
          show();
        }

        return false;
      },
      0);

  auto win = gobj();
  gtk_layer_init_for_window(win);
  gtk_layer_set_layer(win, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(win, "LayerShell/Launcher");
  gtk_layer_set_keyboard_mode(win, GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  input.signal_activate().connect([this]() {
    layer_shell_io::layer_shell_io_app_list_exec_selected(ctx);
    toggle_and_reset();
  });
  input.signal_changed().connect([this]() {
    auto search = input.get_text();
    layer_shell_io::layer_shell_io_app_list_set_search(search.c_str(), ctx);
  });

  auto ctrl = Gtk::EventControllerKey::create();
  ctrl->signal_key_pressed().connect(
      [this](guint keyval, guint, Gdk::ModifierType) {
        std::string key(gdk_keyval_name(keyval));

        if (key == "Escape") {
          toggle_and_reset();
        } else if (key == "Up") {
          layer_shell_io::layer_shell_io_app_list_go_up(ctx);
        } else if (key == "Down") {
          layer_shell_io::layer_shell_io_app_list_go_down(ctx);
        }

        return false;
      },
      false);
  ctrl->set_propagation_phase(Gtk::PropagationPhase::CAPTURE);
  add_controller(ctrl);
}

void Launcher::on_app_list_event(layer_shell_io::Event::AppList_Body data) {
  auto apps = data.apps;
  for (size_t i = 0; i < 5; i++) {
    auto &row = rows.at(i);
    if (i < apps.len) {
      row.update(apps.ptr[i]);
    } else {
      row.hide();
    }
  }
}
void Launcher::on_toggle_launcher_event() { toggle_and_reset(); }

} // namespace windows
