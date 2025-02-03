#include "time.h"
#include "bindings.h"
#include <gtk/gtk.h>

#define _(name) time_widget_ns_##name

static GtkWidget *_(widget);
static GtkWidget *_(label);

static GtkWidget *_(init)(void) {
  _(widget) = gtk_center_box_new();
  gtk_widget_add_css_class(_(widget), "widget");
  gtk_widget_add_css_class(_(widget), "clock");
  gtk_widget_add_css_class(_(widget), "padded");
  gtk_widget_set_name(_(widget), "Time");

  _(label) = gtk_label_new("--");
  gtk_center_box_set_center_widget(GTK_CENTER_BOX(_(widget)), _(label));

  return _(widget);
}

static void _(on_io_event)(const IO_Event *event) {
  switch (event->tag) {
  case IO_Event_Time: {
    gtk_label_set_label(GTK_LABEL(_(label)), event->time.time);
    gtk_widget_set_tooltip_text(_(label), event->time.date);
    break;
  }

  default:
    break;
  }
}

static void _(activate)(void) { layer_shell_io_subscribe(_(on_io_event)); }

widget_t TIME_WIDGET = {.init = _(init), .activate = _(activate)};
