#include "ui/base_overlay.h"
#include <gtk4-layer-shell.h>
#include <vte/vte.h>

typedef struct {
  IOModel *model;
  char *layer_namespace;
  GtkLayerShellLayer layer;
  gboolean auto_exclusive_zone;
  gboolean anchor_top;
  gboolean anchor_left;
  gboolean anchor_right;
  gboolean anchor_bottom;
  gint margin_top;
  gint margin_right;
  gint margin_bottom;
  gint margin_left;
  gboolean keyboard_exclusive;
  gboolean escape_toggle;
  int anchor_bottom_margin;
} BaseOverlayPrivate;

G_DEFINE_TYPE_WITH_PRIVATE(BaseOverlay, base_overlay, GTK_TYPE_WINDOW)

enum {
  PROP_MODEL = 1,
  PROP_LAYER_NAMESPACE,
  PROP_LAYER,
  PROP_AUTO_EXCLUSIVE_ZONE,
  PROP_ANCHOR_TOP,
  PROP_ANCHOR_LEFT,
  PROP_ANCHOR_RIGHT,
  PROP_ANCHOR_BOTTOM,
  PROP_MARGIN_TOP,
  PROP_MARGIN_RIGHT,
  PROP_MARGIN_BOTTOM,
  PROP_MARGIN_LEFT,
  PROP_KEYBOARD_EXCLUSIVE,
  PROP_ESCAPE_TOGGLE,
  PROP_ANCHOR_BOTTOM_MARGIN,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static const char *layer_to_string(GtkLayerShellLayer layer) {
  switch (layer) {
  case GTK_LAYER_SHELL_LAYER_BACKGROUND:
    return "background";
  case GTK_LAYER_SHELL_LAYER_BOTTOM:
    return "bottom";
  case GTK_LAYER_SHELL_LAYER_TOP:
    return "top";
  case GTK_LAYER_SHELL_LAYER_OVERLAY:
    return "overlay";
  default:
    return "overlay";
  }
}

static GtkLayerShellLayer layer_from_string(const char *layer) {
  if (!layer || strcmp(layer, "overlay") == 0) {
    return GTK_LAYER_SHELL_LAYER_OVERLAY;
  }
  if (strcmp(layer, "background") == 0) {
    return GTK_LAYER_SHELL_LAYER_BACKGROUND;
  }
  if (strcmp(layer, "bottom") == 0) {
    return GTK_LAYER_SHELL_LAYER_BOTTOM;
  }
  if (strcmp(layer, "top") == 0) {
    return GTK_LAYER_SHELL_LAYER_TOP;
  }
  g_warning("Unknown layer value '%s', defaulting to 'overlay'", layer);
  return GTK_LAYER_SHELL_LAYER_OVERLAY;
}

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
  case PROP_LAYER:
    g_value_set_string(value, layer_to_string(priv->layer));
    break;
  case PROP_AUTO_EXCLUSIVE_ZONE:
    g_value_set_boolean(value, priv->auto_exclusive_zone);
    break;
  case PROP_ANCHOR_TOP:
    g_value_set_boolean(value, priv->anchor_top);
    break;
  case PROP_ANCHOR_LEFT:
    g_value_set_boolean(value, priv->anchor_left);
    break;
  case PROP_ANCHOR_RIGHT:
    g_value_set_boolean(value, priv->anchor_right);
    break;
  case PROP_ANCHOR_BOTTOM:
    g_value_set_boolean(value, priv->anchor_bottom);
    break;
  case PROP_MARGIN_TOP:
    g_value_set_int(value, priv->margin_top);
    break;
  case PROP_MARGIN_RIGHT:
    g_value_set_int(value, priv->margin_right);
    break;
  case PROP_MARGIN_BOTTOM:
    g_value_set_int(value, priv->margin_bottom);
    break;
  case PROP_MARGIN_LEFT:
    g_value_set_int(value, priv->margin_left);
    break;
  case PROP_KEYBOARD_EXCLUSIVE:
    g_value_set_boolean(value, priv->keyboard_exclusive);
    break;
  case PROP_ESCAPE_TOGGLE:
    g_value_set_boolean(value, priv->escape_toggle);
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
  case PROP_LAYER:
    priv->layer = layer_from_string(g_value_get_string(value));
    break;
  case PROP_AUTO_EXCLUSIVE_ZONE:
    priv->auto_exclusive_zone = g_value_get_boolean(value);
    break;
  case PROP_ANCHOR_TOP:
    priv->anchor_top = g_value_get_boolean(value);
    break;
  case PROP_ANCHOR_LEFT:
    priv->anchor_left = g_value_get_boolean(value);
    break;
  case PROP_ANCHOR_RIGHT:
    priv->anchor_right = g_value_get_boolean(value);
    break;
  case PROP_ANCHOR_BOTTOM:
    priv->anchor_bottom = g_value_get_boolean(value);
    break;
  case PROP_MARGIN_TOP:
    priv->margin_top = g_value_get_int(value);
    break;
  case PROP_MARGIN_RIGHT:
    priv->margin_right = g_value_get_int(value);
    break;
  case PROP_MARGIN_BOTTOM:
    priv->margin_bottom = g_value_get_int(value);
    break;
  case PROP_MARGIN_LEFT:
    priv->margin_left = g_value_get_int(value);
    break;
  case PROP_KEYBOARD_EXCLUSIVE:
    priv->keyboard_exclusive = g_value_get_boolean(value);
    break;
  case PROP_ESCAPE_TOGGLE:
    priv->escape_toggle = g_value_get_boolean(value);
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
  gtk_layer_set_layer(GTK_WINDOW(self), priv->layer);

  if (priv->layer_namespace)
    gtk_layer_set_namespace(GTK_WINDOW(self), priv->layer_namespace);

  if (priv->auto_exclusive_zone)
    gtk_layer_auto_exclusive_zone_enable(GTK_WINDOW(self));

  gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_TOP,
                       priv->anchor_top);
  gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_LEFT,
                       priv->anchor_left);
  gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_RIGHT,
                       priv->anchor_right);
  gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_BOTTOM,
                       priv->anchor_bottom);

  gtk_layer_set_margin(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_TOP,
                       priv->margin_top);
  gtk_layer_set_margin(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_RIGHT,
                       priv->margin_right);
  gtk_layer_set_margin(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_BOTTOM,
                       priv->margin_bottom);
  gtk_layer_set_margin(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_LEFT,
                       priv->margin_left);

  if (priv->keyboard_exclusive)
    gtk_layer_set_keyboard_mode(GTK_WINDOW(self),
                                GTK_LAYER_SHELL_KEYBOARD_MODE_EXCLUSIVE);

  if (priv->anchor_bottom_margin > 0) {
    gtk_layer_set_anchor(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_BOTTOM, true);
    gtk_layer_set_margin(GTK_WINDOW(self), GTK_LAYER_SHELL_EDGE_BOTTOM,
                         priv->anchor_bottom_margin);
  }

  if (priv->escape_toggle) {
    GtkEventController *ctrl = gtk_event_controller_key_new();
    g_signal_connect(ctrl, "key_pressed", G_CALLBACK(key_pressed), self);
    gtk_event_controller_set_propagation_phase(ctrl, GTK_PHASE_CAPTURE);
    gtk_widget_add_controller(GTK_WIDGET(self), ctrl);
  }
}

static void base_overlay_finalize(GObject *object) {
  BaseOverlayPrivate *priv =
      base_overlay_get_instance_private(BASE_OVERLAY(object));
  g_clear_object(&priv->model);
  g_free(priv->layer_namespace);
  G_OBJECT_CLASS(base_overlay_parent_class)->finalize(object);
}

static void base_overlay_init(BaseOverlay *self) {
  BaseOverlayPrivate *priv = base_overlay_get_instance_private(self);
  priv->layer = GTK_LAYER_SHELL_LAYER_OVERLAY;
  priv->auto_exclusive_zone = false;
  priv->anchor_top = false;
  priv->anchor_left = false;
  priv->anchor_right = false;
  priv->anchor_bottom = false;
  priv->margin_top = 0;
  priv->margin_right = 0;
  priv->margin_bottom = 0;
  priv->margin_left = 0;
  priv->keyboard_exclusive = false;
  priv->escape_toggle = true;
  priv->anchor_bottom_margin = 0;
}

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
  properties[PROP_LAYER] =
      g_param_spec_string("layer", NULL, NULL, "overlay", G_PARAM_READWRITE);
  properties[PROP_AUTO_EXCLUSIVE_ZONE] = g_param_spec_boolean(
      "auto-exclusive-zone", NULL, NULL, false, G_PARAM_READWRITE);
  properties[PROP_ANCHOR_TOP] =
      g_param_spec_boolean("anchor-top", NULL, NULL, false, G_PARAM_READWRITE);
  properties[PROP_ANCHOR_LEFT] =
      g_param_spec_boolean("anchor-left", NULL, NULL, false, G_PARAM_READWRITE);
  properties[PROP_ANCHOR_RIGHT] = g_param_spec_boolean(
      "anchor-right", NULL, NULL, false, G_PARAM_READWRITE);
  properties[PROP_ANCHOR_BOTTOM] = g_param_spec_boolean(
      "anchor-bottom", NULL, NULL, false, G_PARAM_READWRITE);
  properties[PROP_MARGIN_TOP] = g_param_spec_int(
      "margin-top", NULL, NULL, 0, G_MAXINT, 0, G_PARAM_READWRITE);
  properties[PROP_MARGIN_RIGHT] = g_param_spec_int(
      "margin-right", NULL, NULL, 0, G_MAXINT, 0, G_PARAM_READWRITE);
  properties[PROP_MARGIN_BOTTOM] = g_param_spec_int(
      "margin-bottom", NULL, NULL, 0, G_MAXINT, 0, G_PARAM_READWRITE);
  properties[PROP_MARGIN_LEFT] = g_param_spec_int(
      "margin-left", NULL, NULL, 0, G_MAXINT, 0, G_PARAM_READWRITE);
  properties[PROP_KEYBOARD_EXCLUSIVE] = g_param_spec_boolean(
      "keyboard-exclusive", NULL, NULL, false, G_PARAM_READWRITE);
  properties[PROP_ESCAPE_TOGGLE] = g_param_spec_boolean(
      "escape-toggle", NULL, NULL, true, G_PARAM_READWRITE);
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
