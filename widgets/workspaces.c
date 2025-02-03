#include "workspaces.h"
#include "bindings.h"
#include <gtk/gtk.h>

#define _(name) workspaces_widget_ns_##name

static GtkWidget *_(widget);
static GtkWidget *_(buttons)[10];

static GtkWidget *_(init)(void) {
  _(widget) = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
  gtk_widget_add_css_class(_(widget), "widget");
  gtk_widget_add_css_class(_(widget), "workspaces");
  gtk_widget_set_name(_(widget), "Workspaces");

  for (size_t i = 0; i < 10; i++) {
    GtkWidget *button = gtk_button_new();
    char buffer[3];
    sprintf(buffer, "%lu", i + 1);
    GtkWidget *label = gtk_label_new(buffer);
    gtk_button_set_child(GTK_BUTTON(button), label);
    gtk_box_append(GTK_BOX(_(widget)), button);
    _(buttons)[i] = button;
  }

  return _(widget);
}

static void _(on_io_event)(const IO_Event *event) {
  switch (event->tag) {
  case IO_Event_Workspaces: {
    for (size_t idx = 1; idx <= 10; idx++) {
      GtkWidget *button = _(buttons)[idx - 1];
      bool visible = false;
      for (size_t i = 0; i < event->workspaces.ids.len; i++) {
        if (event->workspaces.ids.ptr[i] == idx) {
          visible = true;
        }
      }
      gtk_widget_set_visible(button, visible || idx <= 5);
      gtk_widget_remove_css_class(button, "active");
      gtk_widget_remove_css_class(button, "inactive");
      if (idx == event->workspaces.active_id) {
        gtk_widget_add_css_class(button, "active");
      } else {
        gtk_widget_add_css_class(button, "inactive");
      }
    }
    break;
  }
  default: {
    break;
  }
  }
}

static void _(button_on_click)(GtkButton *, gpointer data) {
  size_t idx = (size_t)data;
  layer_shell_io_publish((IO_Command){.tag = IO_Command_HyprlandGoToWorkspace,
                                      .hyprland_go_to_workspace = {idx}});
}

static void _(activate)(void) {
  for (size_t idx = 0; idx < 10; idx++) {
    GtkWidget *button = _(buttons)[idx];
    g_signal_connect(button, "clicked", G_CALLBACK(_(button_on_click)),
                     (void *)idx);
  }

  layer_shell_io_subscribe(_(on_io_event));
}

widget_t WORKSPACES_WIDGET = {.init = _(init), .activate = _(activate)};
