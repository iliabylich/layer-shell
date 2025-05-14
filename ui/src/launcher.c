#include "ui/include/launcher.h"
#include "ui/include/launcher/row.h"
#include <gtk4-layer-shell.h>

#define ROWS_COUNT 5

struct _Launcher {
  GtkWindow parent_instance;

  GtkWidget *input;
  GtkWidget *scroll;
  GtkWidget *rows[ROWS_COUNT];
};

G_DEFINE_TYPE(Launcher, launcher, GTK_TYPE_WINDOW)

enum {
  EXEC_SELECTED = 0,
  GO_UP,
  GO_DOWN,
  RESET,
  SET_SEARCH,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

static void launcher_class_init(LauncherClass *klass) {
#define SIGNAL(name, signal)                                                   \
  signals[signal] =                                                            \
      g_signal_new(name, G_TYPE_FROM_CLASS(klass), G_SIGNAL_RUN_LAST, 0, NULL, \
                   NULL, NULL, G_TYPE_NONE, 0);

  SIGNAL("exec-selected", EXEC_SELECTED);
  SIGNAL("go-up", GO_UP);
  SIGNAL("go-down", GO_DOWN);
  SIGNAL("reset", RESET);
#undef SIGNAL

  signals[SET_SEARCH] =
      g_signal_new("set-search", G_TYPE_FROM_CLASS(klass), G_SIGNAL_RUN_LAST, 0,
                   NULL, NULL, NULL, G_TYPE_NONE, 1, G_TYPE_STRING);
}

static void launcher_init_layer(GtkWindow *window) {
  gtk_layer_init_for_window(window);
  gtk_layer_set_layer(window, GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(window, "LayerShell/Launcher");
  gtk_layer_set_keyboard_mode(window, GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);
}

static void on_submit(GtkEntry *, Launcher *self) {
  g_signal_emit(self, signals[EXEC_SELECTED], 0);
  launcher_toggle_and_reset(self);
}

static void on_input_changed(GtkEditable *input, Launcher *self) {
  const char *search = gtk_editable_get_text(input);
  g_signal_emit(self, signals[SET_SEARCH], 0, search);
}

static bool on_key_pressed(GtkEventControllerKey *, guint keyval, guint,
                           GdkModifierType, Launcher *self) {
  const char *key = gdk_keyval_name(keyval);
  if (strcmp(key, "Escape") == 0) {
    launcher_toggle_and_reset(self);
  } else if (strcmp(key, "Up") == 0) {
    g_signal_emit(self, signals[GO_UP], 0);
  } else if (strcmp(key, "Down") == 0) {
    g_signal_emit(self, signals[GO_DOWN], 0);
  }
  return false;
}

static void launcher_init(Launcher *self) {
  launcher_init_layer(GTK_WINDOW(self));

  self->input =
      g_object_new(GTK_TYPE_SEARCH_ENTRY,
                   //
                   "css-classes", (const char *[]){"search-box", NULL},
                   //
                   "hexpand", true,
                   //
                   NULL);

  GtkWidget *content = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
  for (size_t i = 0; i < ROWS_COUNT; i++) {
    GtkWidget *row = launcher_row_new();
    self->rows[i] = row;
    gtk_box_append(GTK_BOX(content), row);
  }

  self->scroll =
      g_object_new(GTK_TYPE_SCROLLED_WINDOW,
                   //
                   "css-classes", (const char *[]){"scroll-list", NULL},
                   //
                   "can-focus", false,
                   //
                   "child", content,
                   //
                   NULL);

  GtkWidget *layout =
      g_object_new(GTK_TYPE_BOX,
                   //
                   "orientation", GTK_ORIENTATION_VERTICAL,
                   //
                   "spacing", 0,
                   //
                   "css-classes", (const char *[]){"wrapper", NULL},
                   //
                   NULL);

  gtk_box_append(GTK_BOX(layout), self->input);
  gtk_box_append(GTK_BOX(layout), self->scroll);

  gtk_window_set_child(GTK_WINDOW(self), layout);

  g_signal_connect(self->input, "activate", G_CALLBACK(on_submit), self);
  g_signal_connect(self->input, "changed", G_CALLBACK(on_input_changed), self);

  GtkEventController *ctrl = gtk_event_controller_key_new();
  g_signal_connect(ctrl, "key_pressed", G_CALLBACK(on_key_pressed), self);
  gtk_event_controller_set_propagation_phase(ctrl, GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(self), ctrl);
}

GtkWidget *launcher_new(GtkApplication *app) {
  return g_object_new(LAUNCHER_TYPE,
                      //
                      "application", app,
                      //
                      "name", "LauncherWindow",
                      //
                      "width-request", 700,
                      //
                      "height-request", -1,
                      //
                      "css-classes", (const char *[]){"launcher-window", NULL},
                      //
                      NULL);
}

void launcher_refresn(Launcher *self, IO_CArray_LauncherApp apps) {
  for (size_t i = 0; i < ROWS_COUNT; i++) {
    GtkWidget *row = self->rows[i];
    if (i < apps.len) {
      gtk_widget_set_visible(row, true);
      launcher_row_update(LAUNCHER_ROW(row), apps.ptr[i]);
    } else {
      gtk_widget_set_visible(row, false);
    }
  }
}

void launcher_toggle_and_reset(Launcher *self) {
  if (gtk_widget_get_visible(GTK_WIDGET(self))) {
    gtk_widget_set_visible(GTK_WIDGET(self), false);
  } else {
    g_signal_emit(self, signals[RESET], 0);
    gtk_editable_set_text(GTK_EDITABLE(self->input), "");
    gtk_widget_set_visible(GTK_WIDGET(self), true);
  }
}
