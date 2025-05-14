#include "ui/include/base_window.h"
#include "glib-object.h"
#include "vte/vte.h"
#include <gtk4-layer-shell.h>

G_DEFINE_TYPE(BaseWindow, base_window, GTK_TYPE_WINDOW)

enum {
  PROP_0,
  PROP_TOGGLE_ON_ESCAPE,
  PROP_LAYER,
  PROP_LAYER_NAMESPACE,
  PROP_LAYER_KEYBOARD_MODE,
  PROP_LAYER_AUTO_EXCLUSIVE_ZONE_ENABLED,

  PROP_LAYER_ANCHOR_TOP,
  PROP_LAYER_ANCHOR_RIGHT,
  PROP_LAYER_ANCHOR_BOTTOM,
  PROP_LAYER_ANCHOR_LEFT,

  PROP_LAYER_MARGIN_TOP,
  PROP_LAYER_MARGIN_RIGHT,
  PROP_LAYER_MARGIN_BOTTOM,
  PROP_LAYER_MARGIN_LEFT,

  PROP_VTE_COMMAND,
};

void window_toggle(GtkWindow *window) {
  gtk_widget_set_visible(GTK_WIDGET(window),
                         !gtk_widget_get_visible(GTK_WIDGET(window)));
}

static bool on_key_pressed(GtkEventControllerKey *, guint keyval, guint,
                           GdkModifierType, GtkWindow *window) {
  const char *keyname = gdk_keyval_name(keyval);
  if (strcmp(keyname, "Escape") == 0) {
    window_toggle(window);
    return true;
  } else {
    return false;
  }
}

static void set_toggle_on_escape(BaseWindow *self) {
  GtkEventController *ctrl = gtk_event_controller_key_new();
  g_signal_connect(ctrl, "key_pressed", G_CALLBACK(on_key_pressed), self);
  gtk_event_controller_set_propagation_phase(ctrl, GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(self), ctrl);
}

static void set_layer(BaseWindow *self, int layer) {
  gtk_layer_init_for_window(GTK_WINDOW(self));
  gtk_layer_set_layer(GTK_WINDOW(self), layer);
}

static void set_layer_namespace(BaseWindow *self, const char *namespace) {
  gtk_layer_set_namespace(GTK_WINDOW(self), namespace);
}

static void set_keyboard_mode(BaseWindow *self, int keyboard_mode) {
  gtk_layer_set_keyboard_mode(GTK_WINDOW(self), keyboard_mode);
}

static void make_vte_window(BaseWindow *self, char **command) {
  gtk_widget_add_css_class(GTK_WIDGET(self), "terminal-window");

  GtkWidget *terminal = vte_terminal_new();
  vte_terminal_spawn_async(VTE_TERMINAL(terminal), VTE_PTY_DEFAULT,
                           getenv("HOME"), command, NULL, G_SPAWN_DEFAULT, NULL,
                           NULL, NULL, -1, NULL, NULL, NULL);
  gtk_window_set_child(GTK_WINDOW(self), terminal);
}

static void base_window_set_property(GObject *object, guint property_id,
                                     const GValue *value, GParamSpec *pspec) {
  BaseWindow *self = BASE_WINDOW(object);

  switch (property_id) {
  case PROP_TOGGLE_ON_ESCAPE:
    set_toggle_on_escape(self);
    break;

  case PROP_LAYER:
    set_layer(self, g_value_get_int(value));
    break;

  case PROP_LAYER_NAMESPACE:
    set_layer_namespace(self, g_value_get_string(value));
    break;

  case PROP_LAYER_KEYBOARD_MODE:
    set_keyboard_mode(self, g_value_get_int(value));
    break;

  case PROP_LAYER_AUTO_EXCLUSIVE_ZONE_ENABLED:
    gtk_layer_auto_exclusive_zone_enable(GTK_WINDOW(self));
    break;

  case PROP_LAYER_ANCHOR_TOP:
    gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_TOP, true);
    break;

  case PROP_LAYER_ANCHOR_RIGHT:
    gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_RIGHT, true);
    break;

  case PROP_LAYER_ANCHOR_BOTTOM:
    gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_BOTTOM, true);
    break;

  case PROP_LAYER_ANCHOR_LEFT:
    gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_LEFT, true);
    break;

  case PROP_LAYER_MARGIN_TOP:
    gtk_layer_set_margin(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_TOP,
                         g_value_get_int(value));
    break;

  case PROP_LAYER_MARGIN_LEFT:
    gtk_layer_set_margin(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_LEFT,
                         g_value_get_int(value));
    break;

  case PROP_LAYER_MARGIN_BOTTOM:
    gtk_layer_set_margin(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_BOTTOM,
                         g_value_get_int(value));
    break;

  case PROP_LAYER_MARGIN_RIGHT:
    gtk_layer_set_margin(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_RIGHT,
                         g_value_get_int(value));
    break;

  case PROP_VTE_COMMAND:
    char **vte_command = g_value_get_boxed(value);
    make_vte_window(self, vte_command);
    break;

  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void base_window_class_init(BaseWindowClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);

  object_class->set_property = base_window_set_property;

  g_object_class_install_property(
      object_class, PROP_TOGGLE_ON_ESCAPE,
      g_param_spec_boolean("toggle-on-escape", "toggle-on-escape",
                           "toggle-on-escape", false, G_PARAM_WRITABLE));

  g_object_class_install_property(
      object_class, PROP_LAYER,
      g_param_spec_int("layer", "layer", "layer", 0,
                       GTK_LAYER_SHELL_LAYER_ENTRY_NUMBER, 0,
                       G_PARAM_WRITABLE));

  g_object_class_install_property(
      object_class, PROP_LAYER_NAMESPACE,
      g_param_spec_string("layer-namespace", "layer-namespace",
                          "layer-namespace", "", G_PARAM_WRITABLE));

  g_object_class_install_property(
      object_class, PROP_LAYER_KEYBOARD_MODE,
      g_param_spec_int(
          "layer-keyboard-mode", "layer-keyboard-mode", "layer-keyboard-mode",
          0, GTK_LAYER_SHELL_KEYBOARD_MODE_ENTRY_NUMBER, 0, G_PARAM_WRITABLE));

  g_object_class_install_property(
      object_class, PROP_LAYER_AUTO_EXCLUSIVE_ZONE_ENABLED,
      g_param_spec_boolean("layer-auto-exclusive-zone-enabled",
                           "layer-auto-exclusive-zone-enabled",
                           "layer-auto-exclusive-zone-enabled", false,
                           G_PARAM_WRITABLE));

  // anchors

  g_object_class_install_property(
      object_class, PROP_LAYER_ANCHOR_TOP,
      g_param_spec_boolean("layer-anchor-top", "layer-anchor-top",
                           "layer-anchor-top", false, G_PARAM_WRITABLE));

  g_object_class_install_property(
      object_class, PROP_LAYER_ANCHOR_RIGHT,
      g_param_spec_boolean("layer-anchor-right", "layer-anchor-right",
                           "layer-anchor-right", false, G_PARAM_WRITABLE));

  g_object_class_install_property(
      object_class, PROP_LAYER_ANCHOR_BOTTOM,
      g_param_spec_boolean("layer-anchor-bottom", "layer-anchor-bottom",
                           "layer-anchor-bottom", false, G_PARAM_WRITABLE));

  g_object_class_install_property(
      object_class, PROP_LAYER_ANCHOR_LEFT,
      g_param_spec_boolean("layer-anchor-left", "layer-anchor-left",
                           "layer-anchor-left", false, G_PARAM_WRITABLE));

  // margins

  g_object_class_install_property(
      object_class, PROP_LAYER_MARGIN_TOP,
      g_param_spec_int("layer-margin-top", "layer-margin-top",
                       "layer-margin-top", 0, 100000, 0, G_PARAM_WRITABLE));

  g_object_class_install_property(
      object_class, PROP_LAYER_MARGIN_RIGHT,
      g_param_spec_int("layer-margin-right", "layer-margin-right",
                       "layer-margin-right", 0, 100000, 0, G_PARAM_WRITABLE));

  g_object_class_install_property(
      object_class, PROP_LAYER_MARGIN_BOTTOM,
      g_param_spec_int("layer-margin-bottom", "layer-margin-bottom",
                       "layer-margin-bottom", 0, 100000, 0, G_PARAM_WRITABLE));

  g_object_class_install_property(
      object_class, PROP_LAYER_MARGIN_LEFT,
      g_param_spec_int("layer-margin-left", "layer-margin-left",
                       "layer-margin-left", 0, 100000, 0, G_PARAM_WRITABLE));

  // vte

  g_object_class_install_property(
      object_class, PROP_VTE_COMMAND,
      g_param_spec_boxed("vte-command", "vte-command", "vte-command",
                         G_TYPE_STRV, G_PARAM_WRITABLE));
}

static void base_window_init(BaseWindow *) {}
