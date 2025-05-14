#include "ui/include/top_bar/workspaces.h"
#include "gtk/gtk.h"
#include "ui/include/top_bar/workspaces_button.h"

#define WORKSPACES_COUNT 10

struct _Workspaces {
  GtkBox parent_instance;

  GtkWidget *buttons[WORKSPACES_COUNT];
};

G_DEFINE_TYPE(Workspaces, workspaces, GTK_TYPE_BOX)

enum {
  SWITCHED = 0,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

static void workspaces_class_init(WorkspacesClass *klass) {
  signals[SWITCHED] =
      g_signal_new("switched", G_TYPE_FROM_CLASS(klass), G_SIGNAL_RUN_LAST, 0,
                   NULL, NULL, NULL, G_TYPE_NONE, 1, G_TYPE_INT);
}

static const char *css_classes[] = {"widget", "workspaces", NULL};

static void workspaces_init(Workspaces *self) {
  gtk_orientable_set_orientation(GTK_ORIENTABLE(self),
                                 GTK_ORIENTATION_HORIZONTAL);
  gtk_box_set_spacing(GTK_BOX(self), 0);
  gtk_widget_set_css_classes(GTK_WIDGET(self), css_classes);
  gtk_widget_set_name(GTK_WIDGET(self), "Workspaces");

  for (size_t i = 0; i < WORKSPACES_COUNT; i++) {
    GtkWidget *button = workspaces_button_new(self, i);
    gtk_box_append(GTK_BOX(self), button);
    self->buttons[i] = button;
  }
}

GtkWidget *workspaces_new() {
  return g_object_new(workspaces_get_type(), NULL);
}

void workspaces_emit_switched(Workspaces *workspaces, size_t idx) {
  g_signal_emit(workspaces, signals[SWITCHED], 0, idx);
}

void workspaces_refresh(Workspaces *self, IO_CArray_usize ids,
                        size_t active_id) {
  for (size_t i = 0; i < WORKSPACES_COUNT; i++) {
    GtkWidget *button = self->buttons[i];
    bool visible = i < 5;
    for (size_t j = 0; j < ids.len; j++) {
      if (ids.ptr[j] == i + 1) {
        visible = true;
      }
    }
    gtk_widget_set_visible(button, visible);
    if (i + 1 == active_id) {
      workspaces_button_make_active(WORKSPACES_BUTTON(button));
    } else {
      workspaces_button_make_inactive(WORKSPACES_BUTTON(button));
    }
  }
}
