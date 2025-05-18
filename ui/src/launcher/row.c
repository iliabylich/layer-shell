#include "ui/include/launcher/row.h"

typedef struct {
  GtkWidget *image;
  GtkWidget *label;
} data_t;
#define DATA_KEY "data"

GtkWidget *launcher_row_new() {
  GtkWidget *self = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
  gtk_widget_add_css_class(self, "row");

  GtkWidget *image = gtk_image_new();
  gtk_image_set_icon_size(GTK_IMAGE(image), GTK_ICON_SIZE_LARGE);

  GtkWidget *label = gtk_label_new("");
  gtk_label_set_xalign(GTK_LABEL(label), 0.0);
  gtk_widget_set_valign(label, GTK_ALIGN_CENTER);
  gtk_label_set_ellipsize(GTK_LABEL(label), PANGO_ELLIPSIZE_END);

  data_t *data = malloc(sizeof(data_t));
  data->image = image;
  data->label = label;
  g_object_set_data_full(G_OBJECT(self), DATA_KEY, data, free);

  gtk_box_append(GTK_BOX(self), image);
  gtk_box_append(GTK_BOX(self), label);

  return self;
}

void launcher_row_update(GtkWidget *self, IO_LauncherApp app) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);

  if (app.selected) {
    gtk_widget_add_css_class(GTK_WIDGET(self), "active");
  } else {
    gtk_widget_remove_css_class(GTK_WIDGET(self), "active");
  }

  switch (app.icon.tag) {
  case IO_LauncherAppIcon_IconName: {
    gtk_image_set_from_icon_name(GTK_IMAGE(data->image), app.icon.icon_name);
    break;
  }
  case IO_LauncherAppIcon_IconPath: {
    gtk_image_set_from_file(GTK_IMAGE(data->image), app.icon.icon_path);
    break;
  }
  }

  gtk_label_set_label(GTK_LABEL(data->label), app.name);
}
