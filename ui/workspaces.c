#include "ui/workspaces.h"
#include "ui/logger.h"
#include "ui/workspaces_button.h"

LOGGER("Workspaces", 1)

enum signal_types {
  SIGNAL_SWITCHED = 0,
  LAST_SIGNAL,
};
static guint signals[LAST_SIGNAL] = {0};

struct _Workspaces {
  GtkWidget parent_instance;

  GtkWidget *root;
  GList *buttons;
};

G_DEFINE_TYPE(Workspaces, workspaces_widget, GTK_TYPE_WIDGET)

#define WORKSPACES_COUNT 10

static void on_triggered(WorkspacesButton *, guint num, Workspaces *self) {
  g_signal_emit(self, signals[SIGNAL_SWITCHED], 0, num);
}

static void workspaces_widget_init(Workspaces *self) {
  LOG("init");

  self->root = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
  gtk_widget_add_css_class(self->root, "widget");
  gtk_widget_add_css_class(self->root, "workspaces");
  gtk_widget_set_cursor_from_name(self->root, "pointer");

  self->buttons = NULL;
  for (size_t num = 1; num <= WORKSPACES_COUNT; num++) {
    GtkWidget *button = workspaces_button_new(num);
    g_signal_connect(button, "triggered", G_CALLBACK(on_triggered), self);
    gtk_box_append(GTK_BOX(self->root), button);
    self->buttons = g_list_append(self->buttons, button);
  }

  gtk_widget_set_parent(self->root, GTK_WIDGET(self));
}

static void workspaces_widget_dispose(GObject *object) {
  LOG("dispose");

  Workspaces *self = WORKSPACES(object);
  g_clear_pointer(&self->root, gtk_widget_unparent);
  g_clear_pointer(&self->buttons, g_list_free);
  G_OBJECT_CLASS(workspaces_widget_parent_class)->dispose(object);
}

static void workspaces_widget_class_init(WorkspacesClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = workspaces_widget_dispose;

  signals[SIGNAL_SWITCHED] = g_signal_new_class_handler(
      "switched", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST, NULL,
      NULL, NULL, NULL, G_TYPE_NONE, 1, G_TYPE_UINT);

  gtk_widget_class_set_layout_manager_type(GTK_WIDGET_CLASS(klass),
                                           GTK_TYPE_BOX_LAYOUT);
}

GtkWidget *workspaces_new(void) {
  return g_object_new(workspaces_widget_get_type(), NULL);
}

void workspaces_refresh(Workspaces *self,
                        struct IO_FFIArray_HyprlandWorkspace data) {
  size_t i = 0;
  GList *ptr = self->buttons;
  while (ptr != NULL) {
    GtkWidget *button = GTK_WIDGET(ptr->data);
    IO_HyprlandWorkspace workspace = data.ptr[i];

    g_object_set(button, "visible", workspace.visible, "active",
                 workspace.active, NULL);

    i++;
    ptr = ptr->next;
  }
}
