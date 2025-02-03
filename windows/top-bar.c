#include "top-bar.h"
#include "widgets/cpu.h"
#include "widgets/htop.h"
#include "widgets/language.h"
#include "widgets/memory.h"
#include "widgets/network.h"
#include "widgets/session.h"
#include "widgets/sound.h"
#include "widgets/time.h"
#include "widgets/tray.h"
#include "widgets/weather.h"
#include "widgets/workspaces.h"
#include <gtk/gtk.h>
#include <gtk4-layer-shell.h>

#define _(name) top_bar_ns_##name

static GtkWindow *_(window);

static void _(init)(void) {
  _(window) = GTK_WINDOW(gtk_window_new());
  gtk_widget_set_name(GTK_WIDGET(_(window)), "TopBarWindow");

  GtkWidget *layout = gtk_center_box_new();
  gtk_widget_add_css_class(layout, "main-wrapper");
  gtk_window_set_child(_(window), layout);

  GtkWidget *left = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
  GtkWidget *right = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 4);

  gtk_center_box_set_start_widget(GTK_CENTER_BOX(layout), left);
  gtk_center_box_set_end_widget(GTK_CENTER_BOX(layout), right);

  gtk_box_append(GTK_BOX(left), WORKSPACES_WIDGET.init());

  gtk_box_append(GTK_BOX(right), TRAY_WIDGET.init());
  gtk_box_append(GTK_BOX(right), HTOP_WIDGET.init());
  gtk_box_append(GTK_BOX(right), WEATHER_WIDGET.init());
  gtk_box_append(GTK_BOX(right), LANGUAGE_WIDGET.init());
  gtk_box_append(GTK_BOX(right), SOUND_WIDGET.init());
  gtk_box_append(GTK_BOX(right), CPU_WIDGET.init());
  gtk_box_append(GTK_BOX(right), MEMORY_WIDGET.init());
  gtk_box_append(GTK_BOX(right), NETWORK_WIDGET.init());
  gtk_box_append(GTK_BOX(right), TIME_WIDGET.init());
  gtk_box_append(GTK_BOX(right), SESSION_WIDGET.init());
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
  TRAY_WIDGET.activate();
  HTOP_WIDGET.activate();
  WEATHER_WIDGET.activate();
  LANGUAGE_WIDGET.activate();
  SOUND_WIDGET.activate();
  CPU_WIDGET.activate();
  MEMORY_WIDGET.activate();
  NETWORK_WIDGET.activate();
  TIME_WIDGET.activate();
  SESSION_WIDGET.activate();

  gtk_window_present(_(window));
}

window_t TOP_BAR = {
    .init = _(init), .activate = _(activate), .window = _(get_window)};
