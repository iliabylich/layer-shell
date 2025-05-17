#include "ui/include/top_bar/network.h"
#include <ui/include/top_bar/network_popover_action_map.h>

static void emit_settings_clicked(GSimpleAction *, GVariant *,
                                  GtkWidget *parent) {
  network_emit_settings_clicked(parent);
}

static void emit_ping_clicked(GSimpleAction *, GVariant *, GtkWidget *parent) {
  network_emit_ping_clicked(parent);
}

static void emit_network_clicked(GSimpleAction *, GVariant *parameter,
                                 GtkWidget *parent) {
  const char *address = g_variant_get_string(parameter, NULL);
  network_emit_network_clicked(parent, address);
}

static void add(GSimpleActionGroup *action_group, const char *action_name,
                const GVariantType *parameter_type, GCallback callback,
                GtkWidget *parent) {
  GSimpleAction *action = g_simple_action_new(action_name, parameter_type);
  g_signal_connect(action, "activate", callback, parent);
  g_action_map_add_action(G_ACTION_MAP(action_group), G_ACTION(action));
}

GSimpleActionGroup *network_popover_action_map_new(GtkWidget *parent) {
  GSimpleActionGroup *action_group = g_simple_action_group_new();

  add(action_group, NETWORK_SETTINGS_ACTION, NULL,
      G_CALLBACK(emit_settings_clicked), parent);
  add(action_group, NETWORK_PING_ACTION, NULL, G_CALLBACK(emit_ping_clicked),
      parent);
  add(action_group, NETWORK_ROW_ACTION, G_VARIANT_TYPE_STRING,
      G_CALLBACK(emit_network_clicked), parent);

  return action_group;
}
