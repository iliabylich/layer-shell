#include "ui/tracker_row.h"
#include "gtk/gtk.h"
#include "ui/tracker_task.h"

struct _TrackerRow {
  GtkWidget parent_instance;

  GtkWidget *root;
  GtkWidget *title;
  GtkWidget *duration;
};

G_DEFINE_TYPE(TrackerRow, tracker_row, GTK_TYPE_WIDGET)

static void tracker_row_init(TrackerRow *self) {
  self->root = gtk_center_box_new();
  gtk_widget_set_hexpand(self->root, true);
  gtk_widget_add_css_class(GTK_WIDGET(self), "tracker-row");

  self->title = gtk_label_new("<title>");
  gtk_widget_add_css_class(self->title, "title");
  gtk_label_set_justify(GTK_LABEL(self->title), GTK_JUSTIFY_LEFT);
  gtk_center_box_set_start_widget(GTK_CENTER_BOX(self->root), self->title);

  self->duration = gtk_label_new("<duration>");
  gtk_widget_add_css_class(self->duration, "duration");
  gtk_center_box_set_end_widget(GTK_CENTER_BOX(self->root), self->duration);

  gtk_widget_set_parent(self->root, GTK_WIDGET(self));
}

static void tracker_row_dispose(GObject *object) {
  TrackerRow *self = TRACKER_ROW(object);
  g_clear_pointer(&self->root, gtk_widget_unparent);
  G_OBJECT_CLASS(tracker_row_parent_class)->dispose(object);
}

static void tracker_row_class_init(TrackerRowClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = tracker_row_dispose;
  gtk_widget_class_set_layout_manager_type(GTK_WIDGET_CLASS(klass),
                                           GTK_TYPE_BOX_LAYOUT);
}

GtkWidget *tracker_row_new() {
  return g_object_new(tracker_row_get_type(), NULL);
}

void tracker_row_update(TrackerRow *self, TrackerTask *task) {
  gtk_label_set_text(GTK_LABEL(self->title), tracker_task_get_title(task));
  gtk_label_set_text(GTK_LABEL(self->duration),
                     tracker_task_get_duration(task));

  if (tracker_task_get_selected(task)) {
    gtk_widget_add_css_class(GTK_WIDGET(self), "selected");
  } else {
    gtk_widget_remove_css_class(GTK_WIDGET(self), "selected");
  }
}
