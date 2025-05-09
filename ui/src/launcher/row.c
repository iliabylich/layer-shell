#include "ui/include/launcher/row.h"

struct _LauncherRow {
  GtkBox parent_instance;

  GtkWidget *image;
  GtkWidget *label;
};

G_DEFINE_TYPE(LauncherRow, launcher_row, GTK_TYPE_BOX)

static void launcher_row_class_init(LauncherRowClass *) {}

static void launcher_row_init(LauncherRow *self) {
  gtk_orientable_set_orientation(GTK_ORIENTABLE(self),
                                 GTK_ORIENTATION_HORIZONTAL);
  gtk_box_set_spacing(GTK_BOX(self), 0);
  gtk_widget_add_css_class(GTK_WIDGET(self), "row");

  self->image = gtk_image_new();
  gtk_image_set_icon_size(GTK_IMAGE(self->image), GTK_ICON_SIZE_LARGE);

  self->label = gtk_label_new("...");
  gtk_label_set_xalign(GTK_LABEL(self->label), 0.0);
  gtk_widget_set_valign(GTK_WIDGET(self->label), GTK_ALIGN_CENTER);
  gtk_label_set_ellipsize(GTK_LABEL(self->label), PANGO_ELLIPSIZE_END);

  gtk_box_append(GTK_BOX(self), self->image);
  gtk_box_append(GTK_BOX(self), self->label);
}

GtkWidget *launcher_row_new() {
  return g_object_new(launcher_row_get_type(), NULL);
}

void launcher_row_update(LauncherRow *self, IO_LauncherApp app) {
  if (app.selected) {
    gtk_widget_add_css_class(GTK_WIDGET(self), "active");
  } else {
    gtk_widget_remove_css_class(GTK_WIDGET(self), "active");
  }

  switch (app.icon.tag) {
  case IO_LauncherAppIcon_IconName: {
    gtk_image_set_from_icon_name(GTK_IMAGE(self->image), app.icon.icon_name);
    break;
  }
  case IO_LauncherAppIcon_IconPath: {
    gtk_image_set_from_file(GTK_IMAGE(self->image), app.icon.icon_path);
    break;
  }
  }

  gtk_label_set_label(GTK_LABEL(self->label), app.name);
}
