#include "ui/include/top_bar/htop_button.h"

struct _HtopButton {
  GtkButton parent_instance;
};

G_DEFINE_TYPE(HtopButton, htop_button, GTK_TYPE_BUTTON)

static void htop_button_class_init(HtopButtonClass *) {}

static void htop_button_init(HtopButton *) {}

GtkWidget *htop_button_new() {
  return g_object_new(
      HTOP_BUTTON_TYPE,
      //
      "label", "HTop",
      //
      "css-classes",
      (const char *[]){"widget", "terminal", "padded", "clickable", NULL},
      //
      "cursor", gdk_cursor_new_from_name("pointer", NULL),
      //
      "name", "HTop",
      //
      NULL);
}
