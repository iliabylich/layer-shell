#include "ui/base_overlay.h"
#include <gtk4-layer-shell.h>
#include <vte/vte.h>

typedef struct {
  IOModel *model;
  char *layer_namespace;
  gboolean keyboard_exclusive;
  int anchor_bottom_margin;
} BaseOverlayPrivate;

G_DEFINE_TYPE_WITH_PRIVATE(BaseOverlay, base_overlay, GTK_TYPE_WINDOW)

enum {
  PROP_MODEL = 1,
  PROP_LAYER_NAMESPACE,
  PROP_KEYBOARD_EXCLUSIVE,
  PROP_ANCHOR_BOTTOM_MARGIN,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

enum {
  SIGNAL_TOGGLE_REQUESTED = 0,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

static void base_overlay_get_property(GObject *object, guint property_id,
                                      GValue *value, GParamSpec *pspec) {
  BaseOverlayPrivate *priv =
      base_overlay_get_instance_private(BASE_OVERLAY(object));
  switch (property_id) {
  case PROP_MODEL:
    g_value_set_object(value, priv->model);
    break;
  case PROP_LAYER_NAMESPACE:
    g_value_set_string(value, priv->layer_namespace);
    break;
  case PROP_KEYBOARD_EXCLUSIVE:
    g_value_set_boolean(value, priv->keyboard_exclusive);
    break;
  case PROP_ANCHOR_BOTTOM_MARGIN:
    g_value_set_int(value, priv->anchor_bottom_margin);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void base_overlay_set_property(GObject *object, guint property_id,
                                      const GValue *value, GParamSpec *pspec) {
  BaseOverlayPrivate *priv =
      base_overlay_get_instance_private(BASE_OVERLAY(object));
  switch (property_id) {
  case PROP_MODEL:
    g_set_object(&priv->model, g_value_get_object(value));
    break;
  case PROP_LAYER_NAMESPACE:
    g_free(priv->layer_namespace);
    priv->layer_namespace = g_value_dup_string(value);
    break;
  case PROP_KEYBOARD_EXCLUSIVE:
    priv->keyboard_exclusive = g_value_get_boolean(value);
    break;
  case PROP_ANCHOR_BOTTOM_MARGIN:
    priv->anchor_bottom_margin = g_value_get_int(value);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static bool key_pressed(GtkEventControllerKey *, guint keyval, guint,
                        GdkModifierType, BaseOverlay *window) {
  if (strcmp(gdk_keyval_name(keyval), "Escape") == 0) {
    gboolean handled = g_signal_has_handler_pending(
        window, signals[SIGNAL_TOGGLE_REQUESTED], 0, TRUE);
    if (handled) {
      g_signal_emit(window, signals[SIGNAL_TOGGLE_REQUESTED], 0);
    }
    return handled;
  }
  return false;
}

static void base_overlay_constructed(GObject *object) {
  G_OBJECT_CLASS(base_overlay_parent_class)->constructed(object);

  BaseOverlay *self = BASE_OVERLAY(object);
  BaseOverlayPrivate *priv = base_overlay_get_instance_private(self);

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

  GtkEventController *ctrl = gtk_event_controller_key_new();
  g_signal_connect(ctrl, "key_pressed", G_CALLBACK(key_pressed), self);
  gtk_event_controller_set_propagation_phase(ctrl, GTK_PHASE_CAPTURE);
  gtk_widget_add_controller(GTK_WIDGET(self), ctrl);
}

static void base_overlay_finalize(GObject *object) {
  BaseOverlayPrivate *priv =
      base_overlay_get_instance_private(BASE_OVERLAY(object));
  g_clear_object(&priv->model);
  g_free(priv->layer_namespace);
  G_OBJECT_CLASS(base_overlay_parent_class)->finalize(object);
}

static void base_overlay_init(BaseOverlay *) {}

static void base_overlay_class_init(BaseOverlayClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->constructed = base_overlay_constructed;
  object_class->finalize = base_overlay_finalize;
  object_class->get_property = base_overlay_get_property;
  object_class->set_property = base_overlay_set_property;

  properties[PROP_MODEL] =
      g_param_spec_object("model", NULL, NULL, io_model_get_type(),
                          G_PARAM_READWRITE | G_PARAM_CONSTRUCT_ONLY);
  properties[PROP_LAYER_NAMESPACE] = g_param_spec_string(
      "layer-namespace", NULL, NULL, NULL, G_PARAM_READWRITE);
  properties[PROP_KEYBOARD_EXCLUSIVE] = g_param_spec_boolean(
      "keyboard-exclusive", NULL, NULL, false, G_PARAM_READWRITE);
  properties[PROP_ANCHOR_BOTTOM_MARGIN] = g_param_spec_int(
      "anchor-bottom-margin", NULL, NULL, 0, G_MAXINT, 0, G_PARAM_READWRITE);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);

  signals[SIGNAL_TOGGLE_REQUESTED] =
      g_signal_new("toggle-requested", G_TYPE_FROM_CLASS(klass),
                   G_SIGNAL_RUN_LAST, 0, NULL, NULL, NULL, G_TYPE_NONE, 0);
}

void base_overlay_vte(BaseOverlay *self, char **command) {
  gtk_widget_add_css_class(GTK_WIDGET(self), "terminal-overlay");

  GtkWidget *terminal = vte_terminal_new();
  vte_terminal_spawn_async(VTE_TERMINAL(terminal), VTE_PTY_DEFAULT,
                           getenv("HOME"), command, NULL, G_SPAWN_DEFAULT, NULL,
                           NULL, NULL, -1, NULL, NULL, NULL);
  gtk_window_set_child(GTK_WINDOW(self), terminal);
}

IOModel *base_overlay_get_model(BaseOverlay *self) {
  BaseOverlayPrivate *priv = base_overlay_get_instance_private(self);
  return priv->model;
}
