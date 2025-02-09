#include "include/windows/top-bar.hpp"
#include <gtk4-layer-shell.h>

namespace windows {

TopBar::TopBar(const Glib::RefPtr<Gtk::Application> &app, void *ctx) {
  set_name("TopBarWindow");
  set_css_classes({"window"});
  set_application(app);

  Gtk::CenterBox layout;

  layout.add_css_class("main-wrapper");
  set_child(layout);

  Gtk::Box left = Gtk::Box(Gtk::Orientation::HORIZONTAL, 8);
  Gtk::Box right = Gtk::Box(Gtk::Orientation::HORIZONTAL, 4);

  workspaces = new widgets::Workspaces(ctx);
  tray = new widgets::Tray(ctx);
  weather = new widgets::Weather(ctx);
  htop = new widgets::HTop(ctx);
  language = new widgets::Language(ctx);
  sound = new widgets::Sound(ctx);
  cpu = new widgets::CPU(ctx);
  memory = new widgets::Memory(ctx);
  network = new widgets::Network(ctx);
  time = new widgets::Time(ctx);
  session = new widgets::Session(ctx);

  left.append(*workspaces);
  right.append(*tray);
  right.append(*weather);
  right.append(*htop);
  right.append(*language);
  right.append(*sound);
  right.append(*cpu);
  right.append(*memory);
  right.append(*network);
  right.append(*time);
  right.append(*session);

  layout.set_start_widget(left);
  layout.set_end_widget(right);

  auto window = gobj();
  gtk_layer_init_for_window(window);
  gtk_layer_set_layer(window, GTK_LAYER_SHELL_LAYER_TOP);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_anchor(window, GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_margin(window, GTK_LAYER_SHELL_EDGE_TOP, 0);
  gtk_layer_set_namespace(window, "LayerShell/TopBar");

  present();
}

} // namespace windows
