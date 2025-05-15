#include "ui/include/top_bar/power.h"
#include "ui/include/icons.h"
#include "ui/include/macros.h"

struct _Power {
  GtkButton parent_instance;
};

G_DEFINE_TYPE(Power, power, GTK_TYPE_BUTTON)

static void power_class_init(PowerClass *) {}

static void power_init(Power *) {}

GtkWidget *power_new() {
  // clang-format off
  return g_object_new(
      POWER_TYPE,
      "css-classes", CSS("widget", "power", "padded", "clickable"),
      "cursor", gdk_cursor_new_from_name("pointer", NULL),
      "name", "Power",
      "child", gtk_image_new_from_gicon(get_power_icon()), NULL);
  // clang-format on
}
