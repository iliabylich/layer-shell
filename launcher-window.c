#include "launcher-window.h"
#include "bindings.h"
#include <gtk/gtk.h>
#include <gtk4-layer-shell.h>

GtkWindow *launcher_window;
GtkSearchEntry *launcher_input;
typedef struct {
  GtkBox *wrapper;
  GtkImage *icon;
  GtkLabel *label;
} launcher_row_t;
launcher_row_t launcher_rows[5];

void init_launcher_window(void) {
  launcher_window = GTK_WINDOW(gtk_window_new());
  gtk_widget_set_name(GTK_WIDGET(launcher_window), "LauncherWindow");
  GValue width_request = G_VALUE_INIT;
  g_value_init(&width_request, G_TYPE_INT);
  g_value_set_int(&width_request, 700);
  g_object_set_property(G_OBJECT(launcher_window), "width-request",
                        &width_request);

  GtkBox *layout = GTK_BOX(gtk_box_new(GTK_ORIENTATION_VERTICAL, 0));
  gtk_widget_add_css_class(GTK_WIDGET(layout), "widget-launcher-wrapper");
  gtk_window_set_child(launcher_window, GTK_WIDGET(layout));

  launcher_input = GTK_SEARCH_ENTRY(gtk_search_entry_new());
  gtk_widget_add_css_class(GTK_WIDGET(launcher_input),
                           "widget-launcher-search-box");
  gtk_widget_set_hexpand(GTK_WIDGET(launcher_input), true);
  gtk_box_append(layout, GTK_WIDGET(launcher_input));

  GtkScrolledWindow *scroll = GTK_SCROLLED_WINDOW(gtk_scrolled_window_new());
  gtk_widget_add_css_class(GTK_WIDGET(scroll), "widget-launcher-scroll-list");
  gtk_widget_set_can_focus(GTK_WIDGET(scroll), false);
  gtk_box_append(layout, GTK_WIDGET(scroll));

  GtkBox *content = GTK_BOX(gtk_box_new(GTK_ORIENTATION_VERTICAL, 0));
  gtk_scrolled_window_set_child(scroll, GTK_WIDGET(content));

  for (size_t i = 0; i < 5; i++) {
    GtkBox *row = GTK_BOX(gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0));
    gtk_widget_add_css_class(GTK_WIDGET(row), "widget-launcher-row");

    GtkImage *image = GTK_IMAGE(gtk_image_new());
    gtk_image_set_icon_size(image, GTK_ICON_SIZE_LARGE);

    GtkLabel *label = GTK_LABEL(gtk_label_new("..."));
    gtk_label_set_xalign(label, 0.0);
    gtk_widget_set_valign(GTK_WIDGET(label), GTK_ALIGN_CENTER);
    gtk_label_set_ellipsize(label, PANGO_ELLIPSIZE_END);

    gtk_box_append(row, GTK_WIDGET(image));
    gtk_box_append(row, GTK_WIDGET(label));

    gtk_box_append(content, GTK_WIDGET(row));

    launcher_rows[i] =
        (launcher_row_t){.wrapper = row, .icon = image, .label = label};
  }
}

void toggle_launcher_window(void) {
  if (gtk_widget_get_visible(GTK_WIDGET(launcher_window)) == false) {
    layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = AppListReset});
    gtk_editable_set_text(GTK_EDITABLE(launcher_input), "");
  }
  gtk_widget_set_visible(GTK_WIDGET(launcher_window),
                         !gtk_widget_get_visible(GTK_WIDGET(launcher_window)));
}

static void launcher_exec_selected(void) {
  layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = AppListExecSelected});
  toggle_launcher_window();
}

static void launcher_input_changed(GtkEditable *editable) {
  const unsigned char *search =
      (const unsigned char *)gtk_editable_get_text(editable);
  layer_shell_io_publish((LAYER_SHELL_IO_Command){
      .tag = AppListSetSearch, .app_list_set_search = {.search = search}});
}

static gboolean on_launcher_window_key_press(GtkEventControllerKey *self,
                                             guint keyval, guint keycode,
                                             GdkModifierType state,
                                             gpointer user_data) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    toggle_launcher_window();
  } else if (strcmp(gdk_keyval_name(keyval), "Up") == 0) {
    layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = AppListGoUp});
  } else if (strcmp(gdk_keyval_name(keyval), "Down") == 0) {
    layer_shell_io_publish((LAYER_SHELL_IO_Command){.tag = AppListGoDown});
  }

  return false;
}

static void launcher_window_on_event(const LAYER_SHELL_IO_Event *event) {
  switch (event->tag) {
  case ToggleLauncher:
    toggle_launcher_window();
    break;
  case AppList: {
    LAYER_SHELL_IO_CArray_App apps = event->app_list.apps;
    for (size_t i = 0; i < 5; i++) {
      launcher_row_t row = launcher_rows[i];
      if (i < apps.len) {
        LAYER_SHELL_IO_App app = apps.ptr[i];
        gtk_widget_set_visible(GTK_WIDGET(row.wrapper), true);
        if (app.selected) {
          gtk_widget_add_css_class(GTK_WIDGET(row.wrapper), "active");
        } else {
          gtk_widget_remove_css_class(GTK_WIDGET(row.wrapper), "active");
        }

        if (app.icon.tag == IconName) {
          gtk_image_set_from_icon_name(row.icon, app.icon.icon_name.ptr);
        } else {
          gtk_image_set_from_file(row.icon, app.icon.icon_path.ptr);
        }
        gtk_label_set_label(row.label, app.name.ptr);
      } else {
        gtk_widget_set_visible(GTK_WIDGET(row.wrapper), false);
      }
    }
    break;
  }
  default:
    break;
  }
}

void activate_launcher_window(GApplication *app) {
  gtk_window_set_application(launcher_window, GTK_APPLICATION(app));

  gtk_layer_init_for_window(launcher_window);
  gtk_layer_set_layer(launcher_window, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(launcher_window, "LayerShell/Launcher");
  gtk_layer_set_keyboard_mode(launcher_window,
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  g_signal_connect(launcher_input, "activate", launcher_exec_selected, NULL);
  g_signal_connect(launcher_input, "changed",
                   G_CALLBACK(launcher_input_changed), NULL);

  GtkEventControllerKey *ctrl =
      GTK_EVENT_CONTROLLER_KEY(gtk_event_controller_key_new());
  g_signal_connect(ctrl, "key-pressed",
                   G_CALLBACK(on_launcher_window_key_press), NULL);
  gtk_event_controller_set_propagation_phase(GTK_EVENT_CONTROLLER(ctrl),
                                             GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(launcher_window),
                            GTK_EVENT_CONTROLLER(ctrl));

  gtk_window_present(launcher_window);
  gtk_widget_set_visible(GTK_WIDGET(launcher_window), false);

  layer_shell_io_subscribe(launcher_window_on_event);
}
