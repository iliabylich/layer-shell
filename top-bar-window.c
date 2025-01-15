#include "top-bar-window.h"
#include "bindings.h"
#include "cpu-widget.h"
#include "htop-widget.h"
#include "icons.h"
#include "language-widget.h"
#include "memory-widget.h"
#include "network-widget.h"
#include "session-window.h"
#include "sound-widget.h"
#include "weather-widget.h"
#include "workspaces-widget.h"
#include <gtk/gtk.h>
#include <gtk4-layer-shell.h>

#define _(name) top_bar_ns_##name

static GtkWindow *_(window);

static GtkWidget *_(time);
static GtkWidget *_(time_label);

static GtkWidget *_(session);

static void _(init)(void) {
  _(window) = GTK_WINDOW(gtk_window_new());
  gtk_widget_set_name(GTK_WIDGET(_(window)), "TopBarWindow");

  GtkWidget *layout = gtk_center_box_new();
  gtk_widget_add_css_class(layout, "main-wrapper");
  gtk_window_set_child(_(window), layout);

  GtkWidget *left = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
  gtk_center_box_set_start_widget(GTK_CENTER_BOX(layout), left);

  GtkWidget *right = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 4);
  gtk_center_box_set_end_widget(GTK_CENTER_BOX(layout), right);

  // workspaces
  WORKSPACES_WIDGET.init();
  gtk_box_append(GTK_BOX(left), WORKSPACES_WIDGET.main_widget());

  // htop
  HTOP_WIDGET.init();
  gtk_box_append(GTK_BOX(right), HTOP_WIDGET.main_widget());

  // weather
  WEATHER_WIDGET.init();
  gtk_box_append(GTK_BOX(right), WEATHER_WIDGET.main_widget());

  // language
  LANGUAGE_WIDGET.init();
  gtk_box_append(GTK_BOX(right), LANGUAGE_WIDGET.main_widget());

  // sound
  SOUND_WIDGET.init();
  gtk_box_append(GTK_BOX(right), SOUND_WIDGET.main_widget());

  // cpu
  CPU_WIDGET.init();
  gtk_box_append(GTK_BOX(right), CPU_WIDGET.main_widget());

  // ram
  MEMORY_WIDGET.init();
  gtk_box_append(GTK_BOX(right), MEMORY_WIDGET.main_widget());

  // network
  NETWORK_WIDGET.init();
  gtk_box_append(GTK_BOX(right), NETWORK_WIDGET.main_widget());

  // clock
  _(time_label) = gtk_label_new("--");
  _(time) = gtk_center_box_new();
  gtk_widget_add_css_class(_(time), "widget");
  gtk_widget_add_css_class(_(time), "clock");
  gtk_widget_add_css_class(_(time), "padded");
  gtk_center_box_set_center_widget(GTK_CENTER_BOX(_(time)), _(time_label));
  gtk_box_append(GTK_BOX(right), _(time));

  // session
  _(session) = gtk_button_new();
  gtk_widget_add_css_class(_(session), "widget");
  gtk_widget_add_css_class(_(session), "power");
  gtk_widget_add_css_class(_(session), "padded");
  gtk_widget_add_css_class(_(session), "clickable");
  gtk_widget_set_cursor(_(session), gdk_cursor_new_from_name("pointer", NULL));
  GtkWidget *session_image = gtk_image_new();
  gtk_image_set_from_gicon(GTK_IMAGE(session_image), get_power_icon());
  gtk_button_set_child(GTK_BUTTON(_(session)), session_image);
  gtk_box_append(GTK_BOX(right), _(session));
}

static void _(on_io_event)(const LAYER_SHELL_IO_Event *event) {
  switch (event->tag) {
  case Time: {
    gtk_label_set_label(GTK_LABEL(_(time_label)), event->time.time);
    gtk_widget_set_tooltip_text(_(time_label), event->time.date);
    break;
  }

  default:
    break;
  }
}

static GtkWindow *_(get_window)(void) { return _(window); }

static void _(activate)(GApplication *app) {
  gtk_window_set_application(_(window), GTK_APPLICATION(app));

  gtk_layer_init_for_window(_(window));
  gtk_layer_set_layer(_(window), GTK_LAYER_SHELL_LAYER_TOP);
  gtk_layer_set_anchor(_(window), GTK_LAYER_SHELL_EDGE_TOP, true);
  gtk_layer_set_anchor(_(window), GTK_LAYER_SHELL_EDGE_LEFT, true);
  gtk_layer_set_anchor(_(window), GTK_LAYER_SHELL_EDGE_RIGHT, true);
  gtk_layer_set_margin(_(window), GTK_LAYER_SHELL_EDGE_TOP, 0);
  gtk_layer_set_namespace(_(window), "LayerShell/TopBar");

  WORKSPACES_WIDGET.activate();
  HTOP_WIDGET.activate();
  WEATHER_WIDGET.activate();
  LANGUAGE_WIDGET.activate();
  SOUND_WIDGET.activate();
  CPU_WIDGET.activate();
  MEMORY_WIDGET.activate();
  NETWORK_WIDGET.activate();

  g_signal_connect(_(session), "clicked", SESSION.toggle, NULL);

  layer_shell_io_subscribe(_(on_io_event));

  gtk_window_present(_(window));
}

window_t TOP_BAR = {
    .init = _(init), .activate = _(activate), .window = _(get_window)};
