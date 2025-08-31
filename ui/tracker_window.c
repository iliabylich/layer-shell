#include "ui/tracker_window.h"
#include "bindings.h"
#include "gtk/gtk.h"
#include "ui/base_window.h"
#include "ui/logger.h"
#include "ui/tracker_row.h"
#include "ui/tracker_task.h"
#include <gtk4-layer-shell.h>

LOGGER("TrackerWindow", 0)

struct _TrackerWindow {
  GtkWidget parent_instance;

  GtkNoSelection *model;
  size_t count;
  GListStore *store;

  GtkWidget *toggle;

  GtkWidget *root;
};

G_DEFINE_TYPE(TrackerWindow, tracker_window, BASE_WINDOW_TYPE)

enum {
  SIGNAL_ADDED = 0,
  SIGNAL_REMOVED,
  SIGNAL_CUT,
  SIGNAL_SELECTED,
  SIGNAL_TOGGLED,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

static void on_cut(GtkWidget *, TrackerWindow *self) {
  g_signal_emit(self, signals[SIGNAL_CUT], 0);
}

static void on_toggle(GtkWidget *, TrackerWindow *self) {
  g_signal_emit(self, signals[SIGNAL_TOGGLED], 0);
}

static void on_submit(GtkText *text, TrackerWindow *self) {
  const char *title = gtk_editable_get_text(GTK_EDITABLE(text));
  if (strlen(title) == 0) {
    return;
  }
  g_signal_emit(self, signals[SIGNAL_ADDED], 0, title);
  gtk_editable_set_text(GTK_EDITABLE(text), "");
}

static void on_select(GtkListView *, guint position, TrackerWindow *self) {
  void *task = g_list_model_get_item(G_LIST_MODEL(self->store), position);
  const char *uuid = tracker_task_get_uuid(TRACKER_TASK(task));
  g_signal_emit(self, signals[SIGNAL_SELECTED], 0, uuid);
}

static void factory_setup(GtkSignalListItemFactory *, GObject *item, gpointer) {
  GtkWidget *row = tracker_row_new();
  gtk_list_item_set_child(GTK_LIST_ITEM(item), row);
}

static void factory_bind(GtkSignalListItemFactory *, GObject *item, gpointer) {
  GtkWidget *row = gtk_list_item_get_child(GTK_LIST_ITEM(item));
  void *task = gtk_list_item_get_item(GTK_LIST_ITEM(item));
  tracker_row_update(TRACKER_ROW(row), TRACKER_TASK(task));
}

static const char *PAUSE = "";
static const char *PLAY = "";

static void tracker_window_init(TrackerWindow *self) {
  LOG("init");

  gtk_layer_init_for_window(GTK_WINDOW(self));
  gtk_layer_set_layer(GTK_WINDOW(self), GTK_LAYER_SHELL_LAYER_OVERLAY);
  gtk_layer_set_namespace(GTK_WINDOW(self), "LayerShell/Tracker");
  gtk_layer_set_keyboard_mode(GTK_WINDOW(self),
                              GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);
  gtk_widget_add_css_class(GTK_WIDGET(self), "tracker-window");

  base_window_set_toggle_on_escape(BASE_WINDOW(self));

  self->root = gtk_box_new(GTK_ORIENTATION_VERTICAL, 10);

  GtkWidget *toprow = gtk_center_box_new();
  gtk_widget_set_hexpand(toprow, true);
  gtk_box_append(GTK_BOX(self->root), toprow);

  GtkWidget *input = gtk_text_new();
  g_signal_connect(input, "activate", G_CALLBACK(on_submit), self);
  gtk_text_set_placeholder_text(GTK_TEXT(input), "Add new");
  gtk_center_box_set_start_widget(GTK_CENTER_BOX(toprow), input);

  GtkWidget *controls = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 10);
  gtk_center_box_set_end_widget(GTK_CENTER_BOX(toprow), controls);

  GtkWidget *cut = gtk_button_new_with_label("");
  g_signal_connect(cut, "clicked", G_CALLBACK(on_cut), self);
  gtk_widget_set_cursor_from_name(cut, "pointer");
  gtk_box_append(GTK_BOX(controls), cut);

  self->toggle = gtk_button_new_with_label(PLAY);
  g_signal_connect(self->toggle, "clicked", G_CALLBACK(on_toggle), self);
  gtk_widget_set_cursor_from_name(self->toggle, "pointer");
  gtk_box_append(GTK_BOX(controls), self->toggle);

  self->count = 0;
  self->store = g_list_store_new(TRACKER_TASK_TYPE);

  self->model = gtk_no_selection_new(G_LIST_MODEL(self->store));

  GtkListItemFactory *factory = gtk_signal_list_item_factory_new();
  g_signal_connect(factory, "setup", G_CALLBACK(factory_setup), NULL);
  g_signal_connect(factory, "bind", G_CALLBACK(factory_bind), NULL);

  GtkWidget *list_view =
      gtk_list_view_new(GTK_SELECTION_MODEL(self->model), factory);
  gtk_list_view_set_single_click_activate(GTK_LIST_VIEW(list_view), true);
  g_signal_connect(list_view, "activate", G_CALLBACK(on_select), self);
  gtk_box_append(GTK_BOX(self->root), list_view);

  gtk_window_set_child(GTK_WINDOW(self), self->root);
}

static void tracker_window_dispose(GObject *object) {
  LOG("dispose");
  G_OBJECT_CLASS(tracker_window_parent_class)->dispose(object);
}

static void tracker_window_class_init(TrackerWindowClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  signals[SIGNAL_ADDED] = g_signal_new_class_handler(
      "added", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST, NULL, NULL,
      NULL, NULL, G_TYPE_NONE, 1, G_TYPE_STRING);
  signals[SIGNAL_REMOVED] = g_signal_new_class_handler(
      "removed", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST, NULL,
      NULL, NULL, NULL, G_TYPE_NONE, 1, G_TYPE_STRING);
  signals[SIGNAL_CUT] = g_signal_new_class_handler(
      "cut", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST, NULL, NULL,
      NULL, NULL, G_TYPE_NONE, 0);
  signals[SIGNAL_SELECTED] = g_signal_new_class_handler(
      "selected", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST, NULL,
      NULL, NULL, NULL, G_TYPE_NONE, 1, G_TYPE_STRING);
  signals[SIGNAL_TOGGLED] = g_signal_new_class_handler(
      "toggled", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST, NULL,
      NULL, NULL, NULL, G_TYPE_NONE, 0);
  object_class->dispose = tracker_window_dispose;
}

GtkWidget *tracker_window_new(GtkApplication *app) {
  return g_object_new(tracker_window_get_type(), "application", app, NULL);
}

void tracker_window_toggle(TrackerWindow *self) {
  base_window_toggle(BASE_WINDOW(self));
}

static void update_list(TrackerWindow *self, IO_CArray_Task tasks) {
  TrackerTask **new_tasks = malloc(sizeof(TrackerTask *) * tasks.len);
  size_t new_count = tasks.len;
  for (size_t i = 0; i < new_count; i++) {
    IO_Task task = tasks.ptr[i];
    new_tasks[i] =
        tracker_task_new(task.title, task.uuid, task.duration, task.selected);
  }
  g_list_store_splice(self->store, 0, self->count, (void **)new_tasks,
                      new_count);
  self->count = new_count;
  free(new_tasks);
}

static void update_controls(TrackerWindow *self, bool running) {
  if (running) {
    gtk_button_set_label(GTK_BUTTON(self->toggle), PAUSE);
  } else {
    gtk_button_set_label(GTK_BUTTON(self->toggle), PLAY);
  }
}

void tracker_window_refresh(TrackerWindow *self, IO_TrackerUpdatedEvent event) {
  update_list(self, event.view.tasks);
  update_controls(self, event.view.running);
}
