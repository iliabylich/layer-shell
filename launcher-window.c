#include "launcher-window.h"
#include "bindings.h"
#include "utils.h"
#include <gtk/gtk.h>
#include <gtk4-layer-shell.h>

#define ns(name) launcher_ns_##name

GtkWindow *ns(window);

GtkWidget *ns(input);

typedef struct {
  GtkWidget *wrapper;
  GtkWidget *image;
  GtkWidget *label;
} launcher_row_t;
launcher_row_t ns(rows)[5];

static void ns(init)(void) {
  ns(window) = GTK_WINDOW(gtk_window_new());
  gtk_widget_set_name(GTK_WIDGET(ns(window)), "LauncherWindow");

  window_set_width_request(ns(window), 700);

  GtkWidget *layout = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
  gtk_widget_add_css_class(layout, "widget-launcher-wrapper");
  gtk_window_set_child(ns(window), layout);

  ns(input) = gtk_search_entry_new();
  gtk_widget_add_css_class(ns(input), "widget-launcher-search-box");
  gtk_widget_set_hexpand(ns(input), true);
  gtk_box_append(GTK_BOX(layout), ns(input));

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

    ns(rows)[i] =
        (launcher_row_t){.wrapper = row, .image = image, .label = label};
  }
}

static void ns(toggle)(void) {
  if (gtk_widget_get_visible(GTK_WIDGET(ns(window))) == false) {
    layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = AppListReset});
    gtk_editable_set_text(GTK_EDITABLE(ns(input)), "");
  }
  flip_window_visibility(ns(window));
}

static void ns(exec_selected)(void) {
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = AppListExecSelected});
  ns(toggle)();
}

static void ns(on_input_change)(GtkEditable *editable) {
  const unsigned char *search =
      (const unsigned char *)gtk_editable_get_text(editable);
  layer_shell_io_publish((LAYER_SHELL_IO_Command){
      .tag = AppListSetSearch, .app_list_set_search = {.search = search}});
}

static gboolean ns(on_key_press)(GtkEventControllerKey *, guint keyval, guint,
                                 GdkModifierType, gpointer) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    ns(toggle)();
  } else if (strcmp(gdk_keyval_name(keyval), "Up") == 0) {
    layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = AppListGoUp});
  } else if (strcmp(gdk_keyval_name(keyval), "Down") == 0) {
    layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = AppListGoDown});
  }

  return false;
}

static void ns(on_io_event)(const LAYER_SHELL_IO_Event *event) {
  switch (event->tag) {
  case ToggleLauncher:
    ns(toggle)();
    break;
  case AppList: {
    LAYER_SHELL_IO_CArray_App apps = event->app_list.apps;
    for (size_t i = 0; i < 5; i++) {
      launcher_row_t row = ns(rows)[i];
      if (i < apps.len) {
        LAYER_SHELL_IO_App app = apps.ptr[i];
        gtk_widget_set_visible(row.wrapper, true);
        if (app.selected) {
          gtk_widget_add_css_class(row.wrapper, "active");
        } else {
          gtk_widget_remove_css_class(row.wrapper, "active");
        }

        if (app.icon.tag == IconName) {
          gtk_image_set_from_icon_name(GTK_IMAGE(row.image),
                                       app.icon.icon_name.ptr);
        } else {
          gtk_image_set_from_file(GTK_IMAGE(row.image), app.icon.icon_path.ptr);
        }
        gtk_label_set_label(GTK_LABEL(row.label), app.name.ptr);
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

static void ns(activate)(GApplication *app) {
  gtk_window_set_application(ns(window), GTK_APPLICATION(app));

  gtk_layer_init_for_window(ns(window));
  gtk_layer_set_layer(ns(window), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(ns(window), "LayerShell/Launcher");
  gtk_layer_set_keyboard_mode(ns(window),
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  g_signal_connect(ns(input), "activate", ns(exec_selected), NULL);
  g_signal_connect(ns(input), "changed", G_CALLBACK(ns(on_input_change)), NULL);

  GtkEventController *ctrl = gtk_event_controller_key_new();
  g_signal_connect(ctrl, "key-pressed", G_CALLBACK(ns(on_key_press)), NULL);
  gtk_event_controller_set_propagation_phase(ctrl, GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(ns(window)), ctrl);

  gtk_window_present(ns(window));
  gtk_widget_set_visible(GTK_WIDGET(ns(window)), false);

  layer_shell_io_subscribe(ns(on_io_event));
}

window_t LAUNCHER = {
    .init = ns(init), .activate = ns(activate), .toggle = ns(toggle)};
