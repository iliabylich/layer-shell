#include "include/windows/launcher.hpp"
#include <gtk4-layer-shell.h>

namespace windows {

Launcher::Launcher(const Glib::RefPtr<Gtk::Application> &app, io::Ctx *ctx)
    : utils::Subscriber(ctx), rows(5) {
  set_name("LauncherWindow");
  property_width_request().set_value(700);
  set_css_classes({"launcher-window"});
  set_application(app);

  Gtk::Box layout(Gtk::Orientation::VERTICAL, 0);
  layout.set_css_classes({"wrapper"});
  set_child(layout);

  input.set_css_classes({"search-box"});
  input.set_hexpand(true);
  layout.append(input);

  Gtk::ScrolledWindow scroll;
  scroll.set_css_classes({"scroll-list"});
  scroll.set_can_focus(false);
  layout.append(scroll);

  Gtk::Box content(Gtk::Orientation::VERTICAL, 0);
  scroll.set_child(content);

  for (auto &row : rows) {
    content.append(row);
  }

  auto win = gobj();
  gtk_layer_init_for_window(win);
  gtk_layer_set_layer(win, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(win, "LayerShell/Launcher");
  gtk_layer_set_keyboard_mode(win, GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  input.signal_activate().connect([this, ctx]() {
    io::io_launcher_exec_selected(ctx);
    toggle_and_reset();
  });
  input.signal_changed().connect([this, ctx]() {
    auto search = input.get_text();
    io::io_launcher_set_search(search.c_str(), ctx);
  });

  auto ctrl = Gtk::EventControllerKey::create();
  ctrl->signal_key_pressed().connect(
      [this, ctx](guint keyval, guint, Gdk::ModifierType) {
        std::string key(gdk_keyval_name(keyval));

        if (key == "Escape") {
          toggle_and_reset();
        } else if (key == "Up") {
          io::io_launcher_go_up(ctx);
        } else if (key == "Down") {
          io::io_launcher_go_down(ctx);
        }

        return false;
      },
      false);
  ctrl->set_propagation_phase(Gtk::PropagationPhase::CAPTURE);
  add_controller(ctrl);
}

void Launcher::toggle_and_reset() {
  Glib::signal_timeout().connect(
      [this]() {
        // Toggling immediately breaks something in gtkmm, so we
        // schedule it
        if (is_visible()) {
          hide();
        } else {
          io::io_launcher_reset(ctx);
          input.set_text("");
          show();
        }

        return false;
      },
      0);
}

void Launcher::on_io_event(io::Event::Launcher_Body data) {
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

Launcher *Launcher::instance;
void Launcher::init(const Glib::RefPtr<Gtk::Application> &app, io::Ctx *ctx) {
  instance = new Launcher(app, ctx);
}
Launcher *Launcher::get() { return instance; }

} // namespace windows
