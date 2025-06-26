#include "ui/include/top_bar/network_popover_action_map.h"
#include "ui/include/top_bar/network.h"

#define SET_CB(obj, cb)                                                        \
  g_object_set_data(G_OBJECT(obj), "callback", (void *)(size_t)cb);
#define GET_CB(obj, Type)                                                      \
  ((Type)(size_t)g_object_get_data(G_OBJECT(obj), "callback"))

static void settings_row_clicked(GSimpleAction *action) {
  GET_CB(action, network_settings_clicked_f)();
}

static void ping_row_clicked(GSimpleAction *action) {
  GET_CB(action, network_ping_clicked_f)();
}

static void network_row_clicked(GSimpleAction *action, GVariant *parameter) {
  const char *address = g_variant_get_string(parameter, NULL);
  GET_CB(action, network_address_clicked_f)(address);
}

GSimpleActionGroup *
network_popover_action_map_new(network_settings_clicked_f on_settings_clicked,
                               network_ping_clicked_f on_ping_clicked,
                               network_address_clicked_f on_address_clicked) {
  GSimpleActionGroup *action_group = g_simple_action_group_new();
  GSimpleAction *action;

  action = g_simple_action_new("settings-row", NULL);
  SET_CB(action, on_settings_clicked);
  g_signal_connect(action, "activate", G_CALLBACK(settings_row_clicked), NULL);
  g_action_map_add_action(G_ACTION_MAP(action_group), G_ACTION(action));

  action = g_simple_action_new("ping-row", NULL);
  SET_CB(action, on_ping_clicked);
  g_signal_connect(action, "activate", G_CALLBACK(ping_row_clicked), NULL);
  g_action_map_add_action(G_ACTION_MAP(action_group), G_ACTION(action));

  action = g_simple_action_new("network-row", G_VARIANT_TYPE_STRING);
  SET_CB(action, on_address_clicked);
  g_signal_connect(action, "activate", G_CALLBACK(network_row_clicked), NULL);
  g_action_map_add_action(G_ACTION_MAP(action_group), G_ACTION(action));

  return action_group;
}
