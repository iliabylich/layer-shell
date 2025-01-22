#include "launcher.h"
#include "../bindings.h"
#include <gtk/gtk.h>
#include <gtk4-layer-shell.h>

#define _(name) launcher_ns_##name

static GtkWindow *_(window);

static GtkWidget *_(input);

typedef struct {
  GtkWidget *wrapper;
  GtkWidget *image;
  GtkWidget *label;
} row_t;
static row_t _(rows)[5];

static void _(init)(void) {
  _(window) = GTK_WINDOW(gtk_window_new());
  gtk_widget_set_name(GTK_WIDGET(_(window)), "LauncherWindow");

  window_set_width_request(_(window), 700);

  GtkWidget *layout = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
  gtk_widget_add_css_class(layout, "widget-launcher-wrapper");
  gtk_window_set_child(_(window), layout);

  _(input) = gtk_search_entry_new();
  gtk_widget_add_css_class(_(input), "widget-launcher-search-box");
  gtk_widget_set_hexpand(_(input), true);
  gtk_box_append(GTK_BOX(layout), _(input));

  GtkWidget *scroll = gtk_scrolled_window_new();
  gtk_widget_add_css_class(scroll, "widget-launcher-scroll-list");
  gtk_widget_set_can_focus(scroll, false);
  gtk_box_append(GTK_BOX(layout), scroll);

  GtkWidget *content = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
  gtk_scrolled_window_set_child(GTK_SCROLLED_WINDOW(scroll), content);

  for (size_t i = 0; i < 5; i++) {
    GtkWidget *row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
    gtk_widget_add_css_class(row, "widget-launcher-row");

    GtkWidget *image = gtk_image_new();
    gtk_image_set_icon_size(GTK_IMAGE(image), GTK_ICON_SIZE_LARGE);

    GtkWidget *label = gtk_label_new("...");
    gtk_label_set_xalign(GTK_LABEL(label), 0.0);
    gtk_widget_set_valign(label, GTK_ALIGN_CENTER);
    gtk_label_set_ellipsize(GTK_LABEL(label), PANGO_ELLIPSIZE_END);

    gtk_box_append(GTK_BOX(row), image);
    gtk_box_append(GTK_BOX(row), label);

    gtk_box_append(GTK_BOX(content), row);

    _(rows)
    [i] = (row_t){.wrapper = row, .image = image, .label = label};
  }
}

static void _(toggle)(void) {
  if (gtk_widget_get_visible(GTK_WIDGET(_(window))) == false) {
    layer_shell_io_publish((IO_Command){.tag = IO_Command_AppListReset});
    gtk_editable_set_text(GTK_EDITABLE(_(input)), "");
  }
  flip_window_visibility(_(window));
}

static void _(exec_selected)(void) {
  layer_shell_io_publish((IO_Command){.tag = IO_Command_AppListExecSelected});
  _(toggle)();
}

static void _(on_input_change)(GtkEditable *editable) {
  const unsigned char *search =
      (const unsigned char *)gtk_editable_get_text(editable);
  layer_shell_io_publish(
      (IO_Command){.tag = IO_Command_AppListSetSearch,
                   .app_list_set_search = {.search = search}});
}

static gboolean _(on_key_press)(GtkEventControllerKey *, guint keyval, guint,
                                GdkModifierType, gpointer) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    _(toggle)();
  } else if (strcmp(gdk_keyval_name(keyval), "Up") == 0) {
    layer_shell_io_publish((IO_Command){.tag = IO_Command_AppListGoUp});
  } else if (strcmp(gdk_keyval_name(keyval), "Down") == 0) {
    layer_shell_io_publish((IO_Command){.tag = IO_Command_AppListGoDown});
  }

  return false;
}

static void _(on_io_event)(const IO_Event *event) {
  switch (event->tag) {
  case IO_Event_ToggleLauncher:
    _(toggle)();
    break;
  case IO_Event_AppList: {
    IO_CArray_App apps = event->app_list.apps;
    for (size_t i = 0; i < 5; i++) {
      row_t row = _(rows)[i];
      if (i < apps.len) {
        IO_App app = apps.ptr[i];
        gtk_widget_set_visible(row.wrapper, true);
        if (app.selected) {
          gtk_widget_add_css_class(row.wrapper, "active");
        } else {
          gtk_widget_remove_css_class(row.wrapper, "active");
        }

        if (app.icon.tag == IO_AppIcon_IconName) {
          gtk_image_set_from_icon_name(GTK_IMAGE(row.image),
                                       app.icon.icon_name);
        } else {
          gtk_image_set_from_file(GTK_IMAGE(row.image), app.icon.icon_path);
        }
        gtk_label_set_label(GTK_LABEL(row.label), app.name);
      } else {
        gtk_widget_set_visible(row.wrapper, false);
      }
    }
    break;
  }
  default:
    break;
  }
}

static void _(activate)(GApplication *app) {
  gtk_window_set_application(_(window), GTK_APPLICATION(app));

  gtk_layer_init_for_window(_(window));
  gtk_layer_set_layer(_(window), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(_(window), "LayerShell/Launcher");
  gtk_layer_set_keyboard_mode(_(window),
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  g_signal_connect(_(input), "activate", _(exec_selected), NULL);
  g_signal_connect(_(input), "changed", G_CALLBACK(_(on_input_change)), NULL);

  GtkEventController *ctrl = gtk_event_controller_key_new();
  g_signal_connect(ctrl, "key-pressed", G_CALLBACK(_(on_key_press)), NULL);
  gtk_event_controller_set_propagation_phase(ctrl, GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(_(window)), ctrl);

  gtk_window_present(_(window));
  gtk_widget_set_visible(GTK_WIDGET(_(window)), false);

  layer_shell_io_subscribe(_(on_io_event));
}

window_t LAUNCHER = {
    .init = _(init), .activate = _(activate), .toggle = _(toggle)};
