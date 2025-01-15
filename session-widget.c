#include "session-widget.h"
#include "icons.h"
#include "session-window.h"
#include <gtk/gtk.h>

#define _(name) session_widget_ns_##name

static GtkWidget *_(widget);

static GtkWidget *_(init)(void) {
  _(widget) = gtk_button_new();
  gtk_widget_add_css_class(_(widget), "widget");
  gtk_widget_add_css_class(_(widget), "power");
  gtk_widget_add_css_class(_(widget), "padded");
  gtk_widget_add_css_class(_(widget), "clickable");
  gtk_widget_set_cursor(_(widget), gdk_cursor_new_from_name("pointer", NULL));

  GtkWidget *image = gtk_image_new();
  gtk_image_set_from_gicon(GTK_IMAGE(image), get_power_icon());
  gtk_button_set_child(GTK_BUTTON(_(widget)), image);

  return _(widget);
}

static void _(activate)(void) {
  g_signal_connect(_(widget), "clicked", SESSION.toggle, NULL);
}

widget_t SESSION_WIDGET = {.init = _(init), .activate = _(activate)};
