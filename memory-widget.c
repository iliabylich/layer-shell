#include "memory-widget.h"
#include "bindings.h"
#include <gtk/gtk.h>

#define _(name) memory_widget_ns_##name

static GtkWidget *_(widget);
static GtkWidget *_(label);

static void _(init)(void) {
  _(label) = gtk_label_new(NULL);
  _(widget) = gtk_button_new();
  gtk_widget_add_css_class(_(widget), "widget");
  gtk_widget_add_css_class(_(widget), "memory");
  gtk_widget_add_css_class(_(widget), "padded");
  gtk_widget_add_css_class(_(widget), "clickable");
  gtk_button_set_child(GTK_BUTTON(_(widget)), _(label));
}

static void _(spawn_system_monitor)(void) {
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = SpawnSystemMonitor});
}

static void _(on_io_event)(const LAYER_SHELL_IO_Event *event) {
  switch (event->tag) {
  case Memory: {
    char buffer[100];
    sprintf(buffer, "RAM %.1fG/%.1fG", event->memory.used, event->memory.total);
    gtk_label_set_label(GTK_LABEL(_(label)), buffer);
    break;
  }
  default: {
    break;
  }
  }
}

static void _(activate)(void) {
  g_signal_connect(_(widget), "clicked", _(spawn_system_monitor), NULL);

  layer_shell_io_subscribe(_(on_io_event));
}

static GtkWidget *_(main_widget)(void) { return _(widget); }

widget_t MEMORY_WIDGET = {
    .init = _(init), .activate = _(activate), .main_widget = _(main_widget)};
