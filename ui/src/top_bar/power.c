#include "ui/include/top_bar/power.h"
#include "gtk/gtk.h"
#include "ui/include/icons.h"

struct _Power {
  GtkButton parent_instance;
};

G_DEFINE_TYPE(Power, power, GTK_TYPE_BUTTON)

static void power_class_init(PowerClass *) {}

static void power_init(Power *) {}

GtkWidget *power_new() {
  return g_object_new(
      power_get_type(),
      //
      "css-classes",
      (const char *[]){"widget", "power", "padded", "clickable", NULL},
      //
      "cursor", gdk_cursor_new_from_name("pointer", NULL),
      //
      "name", "Power",
      //
      "child", gtk_image_new_from_gicon(get_power_icon()), NULL);
}
