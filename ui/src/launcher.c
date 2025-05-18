#include "ui/include/launcher.h"
#include "gtk/gtk.h"
#include "gtk/gtkshortcut.h"
#include "ui/include/builder.h"
#include "ui/include/launcher/row.h"
#include <gtk4-layer-shell.h>

#define ROWS_COUNT 5
typedef struct {
  GtkWidget *input;
  GtkWidget *scroll;
  GtkWidget *rows[ROWS_COUNT];

  launcher_exec_selected_f launcher_exec_selected_callback;
  launcher_go_up_f launcher_go_up_callback;
  launcher_go_down_f launcher_go_down_callback;
  launcher_reset_f launcher_reset_callback;
  launcher_set_search_f launcher_set_search_callback;
} data_t;
#define DATA_KEY "data"

static void on_submit(GtkEntry *, GtkWidget *self);
static void on_input_changed(GtkEditable *input, GtkWidget *self);
static bool on_key_pressed(GtkEventControllerKey *, guint keyval, guint,
                           GdkModifierType, GtkWidget *self);

GtkWidget *
launcher_init(GtkApplication *app,
              launcher_exec_selected_f launcher_exec_selected_callback,
              launcher_go_up_f launcher_go_up_callback,
              launcher_go_down_f launcher_go_down_callback,
              launcher_reset_f launcher_reset_callback,
              launcher_set_search_f launcher_set_search_callback) {
  GtkWidget *self = launcher_get_widget("LAUNCHER");
  gtk_window_set_application(GTK_WINDOW(self), app);
  gtk_layer_init_for_window(GTK_WINDOW(self));
  gtk_layer_set_layer(GTK_WINDOW(self), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(GTK_WINDOW(self), "LayerShell/Launcher");
  gtk_layer_set_keyboard_mode(GTK_WINDOW(self),
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  GtkWidget *input = launcher_get_widget("INPUT");
  GtkWidget *scroll = launcher_get_widget("SCROLL");
  GtkWidget *content = launcher_get_widget("CONTENT");

  data_t *data = malloc(sizeof(data_t));
  data->input = input;
  data->scroll = scroll;
  data->launcher_exec_selected_callback = launcher_exec_selected_callback;
  data->launcher_go_up_callback = launcher_go_up_callback;
  data->launcher_go_down_callback = launcher_go_down_callback;
  data->launcher_reset_callback = launcher_reset_callback;
  data->launcher_set_search_callback = launcher_set_search_callback;
  g_object_set_data_full(G_OBJECT(self), DATA_KEY, data, free);

  for (size_t i = 0; i < ROWS_COUNT; i++) {
    GtkWidget *row = launcher_row_new();
    data->rows[i] = row;
    gtk_box_append(GTK_BOX(content), row);
  }

  g_signal_connect(input, "activate", G_CALLBACK(on_submit), self);
  g_signal_connect(input, "changed", G_CALLBACK(on_input_changed), self);

  GtkEventController *ctrl = gtk_event_controller_key_new();
  g_signal_connect(ctrl, "key_pressed", G_CALLBACK(on_key_pressed), self);
  gtk_event_controller_set_propagation_phase(ctrl, GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(self), ctrl);

  return self;
}

static void on_submit(GtkEntry *, GtkWidget *self) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);
  data->launcher_exec_selected_callback();
  launcher_toggle_and_reset(self);
}

static void on_input_changed(GtkEditable *input, GtkWidget *self) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);
  const char *search = gtk_editable_get_text(input);
  data->launcher_set_search_callback((const uint8_t *)search);
}

static bool on_key_pressed(GtkEventControllerKey *, guint keyval, guint,
                           GdkModifierType, GtkWidget *self) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);

  const char *key = gdk_keyval_name(keyval);
  if (strcmp(key, "Escape") == 0) {
    launcher_toggle_and_reset(self);
  } else if (strcmp(key, "Up") == 0) {
    data->launcher_go_up_callback();
  } else if (strcmp(key, "Down") == 0) {
    data->launcher_go_down_callback();
  }
  return false;
}

void launcher_refresn(GtkWidget *self, IO_CArray_LauncherApp apps) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);

  for (size_t i = 0; i < ROWS_COUNT; i++) {
    GtkWidget *row = data->rows[i];
    if (i < apps.len) {
      gtk_widget_set_visible(row, true);
      launcher_row_update(row, apps.ptr[i]);
    } else {
      gtk_widget_set_visible(row, false);
    }
  }
}

void launcher_toggle_and_reset(GtkWidget *self) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);

  if (gtk_widget_get_visible(GTK_WIDGET(self))) {
    gtk_widget_set_visible(GTK_WIDGET(self), false);
  } else {
    data->launcher_reset_callback();
    gtk_editable_set_text(GTK_EDITABLE(data->input), "");
    gtk_widget_set_visible(GTK_WIDGET(self), true);
  }
}
