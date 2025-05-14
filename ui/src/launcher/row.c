#include "ui/include/launcher/row.h"

struct _LauncherRow {
  GtkBox parent_instance;

  GtkWidget *image;
  GtkWidget *label;
};

G_DEFINE_TYPE(LauncherRow, launcher_row, GTK_TYPE_BOX)

static void launcher_row_class_init(LauncherRowClass *) {}

static void launcher_row_init(LauncherRow *self) {
  self->image = g_object_new(GTK_TYPE_IMAGE,
                             //
                             "icon-size", GTK_ICON_SIZE_LARGE,
                             //
                             NULL);

  self->label = g_object_new(GTK_TYPE_LABEL,
                             //
                             "xalign", 0.0,
                             //
                             "valign", GTK_ALIGN_CENTER,
                             //
                             "ellipsize", PANGO_ELLIPSIZE_END,
                             //
                             NULL);

  gtk_box_append(GTK_BOX(self), self->image);
  gtk_box_append(GTK_BOX(self), self->label);
}

GtkWidget *launcher_row_new() {
  return g_object_new(launcher_row_get_type(),
                      //
                      "orientation", GTK_ORIENTATION_HORIZONTAL,
                      //
                      "spacing", 0,
                      //
                      "css-classes", (const char *[]){"row", NULL},
                      //
                      NULL);
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
