#include "ui/base_window.h"
#include <gtk4-layer-shell.h>
#include <vte/vte.h>

typedef struct {
  char *layer_namespace;
  gboolean keyboard_exclusive;
  gboolean toggle_on_escape;
  int anchor_bottom_margin;
  WindowModel *window_state;
} BaseWindowPrivate;

G_DEFINE_TYPE_WITH_PRIVATE(BaseWindow, base_window, GTK_TYPE_WINDOW)

enum {
  PROP_LAYER_NAMESPACE = 1,
  PROP_KEYBOARD_EXCLUSIVE,
  PROP_TOGGLE_ON_ESCAPE,
  PROP_ANCHOR_BOTTOM_MARGIN,
  PROP_WINDOW_STATE,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static void base_window_get_property(GObject *object, guint property_id,
                                     GValue *value, GParamSpec *pspec) {
  BaseWindowPrivate *priv =
      base_window_get_instance_private(BASE_WINDOW(object));
  switch (property_id) {
  case PROP_LAYER_NAMESPACE:
    g_value_set_string(value, priv->layer_namespace);
    break;
  case PROP_KEYBOARD_EXCLUSIVE:
    g_value_set_boolean(value, priv->keyboard_exclusive);
    break;
  case PROP_TOGGLE_ON_ESCAPE:
    g_value_set_boolean(value, priv->toggle_on_escape);
    break;
  case PROP_ANCHOR_BOTTOM_MARGIN:
    g_value_set_int(value, priv->anchor_bottom_margin);
    break;
  case PROP_WINDOW_STATE:
    g_value_set_object(value, priv->window_state);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void base_window_set_property(GObject *object, guint property_id,
                                     const GValue *value, GParamSpec *pspec) {
  BaseWindowPrivate *priv =
      base_window_get_instance_private(BASE_WINDOW(object));
  switch (property_id) {
  case PROP_LAYER_NAMESPACE:
    g_free(priv->layer_namespace);
    priv->layer_namespace = g_value_dup_string(value);
    break;
  case PROP_KEYBOARD_EXCLUSIVE:
    priv->keyboard_exclusive = g_value_get_boolean(value);
    break;
  case PROP_TOGGLE_ON_ESCAPE:
    priv->toggle_on_escape = g_value_get_boolean(value);
    break;
  case PROP_ANCHOR_BOTTOM_MARGIN:
    priv->anchor_bottom_margin = g_value_get_int(value);
    break;
  case PROP_WINDOW_STATE:
    g_set_object(&priv->window_state, g_value_get_object(value));
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static bool key_pressed(GtkEventControllerKey *, guint keyval, guint,
                        GdkModifierType, BaseWindow *window) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    base_window_toggle_window_visible(window);
    return true;
  }
  return false;
}

static void base_window_constructed(GObject *object) {
  G_OBJECT_CLASS(base_window_parent_class)->constructed(object);

  BaseWindow *self = BASE_WINDOW(object);
  BaseWindowPrivate *priv = base_window_get_instance_private(self);

  gtk_layer_init_for_window(GTK_WINDOW(self));
  gtk_layer_set_layer(GTK_WINDOW(self), GTK_LAYER_SHELL_LAYER_OVERLAY);

  if (priv->layer_namespace)
    gtk_layer_set_namespace(GTK_WINDOW(self), priv->layer_namespace);

  if (priv->keyboard_exclusive)
    gtk_layer_set_keyboard_mode(GTK_WINDOW(self),
                                GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  if (priv->anchor_bottom_margin > 0) {
    gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_BOTTOM, true);
    gtk_layer_set_margin(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_BOTTOM,
                         priv->anchor_bottom_margin);
  }

  if (priv->toggle_on_escape) {
    GtkEventController *ctrl = gtk_event_controller_key_new();
    g_signal_connect(ctrl, "key_pressed", G_CALLBACK(key_pressed), self);
    gtk_event_controller_set_propagation_phase(ctrl, GTK_PHASE_CAPTURE);
    gtk_widget_add_controller(GTK_WIDGET(self), ctrl);
  }
}

static void base_window_finalize(GObject *object) {
  BaseWindowPrivate *priv =
      base_window_get_instance_private(BASE_WINDOW(object));
  g_free(priv->layer_namespace);
  g_clear_object(&priv->window_state);
  G_OBJECT_CLASS(base_window_parent_class)->finalize(object);
}

static void base_window_init(BaseWindow *) {}

static void base_window_class_init(BaseWindowClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->constructed = base_window_constructed;
  object_class->finalize = base_window_finalize;
  object_class->get_property = base_window_get_property;
  object_class->set_property = base_window_set_property;

  properties[PROP_LAYER_NAMESPACE] = g_param_spec_string(
      "layer-namespace", NULL, NULL, NULL, G_PARAM_READWRITE);
  properties[PROP_KEYBOARD_EXCLUSIVE] = g_param_spec_boolean(
      "keyboard-exclusive", NULL, NULL, false, G_PARAM_READWRITE);
  properties[PROP_TOGGLE_ON_ESCAPE] = g_param_spec_boolean(
      "toggle-on-escape", NULL, NULL, false, G_PARAM_READWRITE);
  properties[PROP_ANCHOR_BOTTOM_MARGIN] = g_param_spec_int(
      "anchor-bottom-margin", NULL, NULL, 0, G_MAXINT, 0, G_PARAM_READWRITE);
  properties[PROP_WINDOW_STATE] =
      g_param_spec_object("window-state", NULL, NULL, window_model_get_type(),
                          G_PARAM_READWRITE | G_PARAM_CONSTRUCT_ONLY);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

void base_window_vte(BaseWindow *self, char **command) {
  gtk_widget_add_css_class(GTK_WIDGET(self), "terminal-window");

  GtkWidget *terminal = vte_terminal_new();
  vte_terminal_spawn_async(VTE_TERMINAL(terminal), VTE_PTY_DEFAULT,
                           getenv("HOME"), command, NULL, G_SPAWN_DEFAULT, NULL,
                           NULL, NULL, -1, NULL, NULL, NULL);
  gtk_window_set_child(GTK_WINDOW(self), terminal);
}

void base_window_set_window_visible(BaseWindow *self, bool visible) {
  BaseWindowPrivate *priv = base_window_get_instance_private(self);
  if (priv->window_state) {
    g_object_set(priv->window_state, "visible", visible, NULL);
  } else {
    gtk_widget_set_visible(GTK_WIDGET(self), visible);
  }
}

void base_window_toggle_window_visible(BaseWindow *self) {
  BaseWindowPrivate *priv = base_window_get_instance_private(self);
  if (priv->window_state) {
    gboolean visible = false;
    g_object_get(priv->window_state, "visible", &visible, NULL);
    g_object_set(priv->window_state, "visible", !visible, NULL);
  } else {
    gtk_widget_set_visible(GTK_WIDGET(self),
                           !gtk_widget_get_visible(GTK_WIDGET(self)));
  }
}
