#include "ui/include/htop.h"
#include "ui/include/macros.h"
#include <gtk4-layer-shell.h>

struct _Htop {
  BaseWindow parent_instance;
};

G_DEFINE_TYPE(Htop, htop, BASE_WINDOW_TYPE)

static void htop_class_init(HtopClass *) {}

static void htop_init(Htop *) {}

GtkWidget *htop_new(GtkApplication *app) {
  // clang-format off
  return g_object_new(
      HTOP_TYPE,
      "application", app,
      "name", "HtopWindow",
      "width-request", 1000,
      "height-request", 700,
      "toggle-on-escape", true,
      "layer", GTK_LAYER_SHELL_LAYER_OVERLAY,
      "layer-namespace", "LayerShell/Htop",
      "layer-keyboard-mode", GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE,
      "vte-command", VTE_CMD("htop"),
      NULL);
  // clang-format on
}
