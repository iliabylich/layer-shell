#include "ui/include/top_bar/network_popover_menu.h"

static void add(GMenu *menu, const char *label, const char *action,
                GVariant *target_value) {
  GMenuItem *item = g_menu_item_new(label, NULL);
  g_menu_item_set_action_and_target_value(item, action, target_value);
  g_menu_append_item(menu, item);
  g_object_unref(item);
}

void network_popover_menu_add_settings(GMenu *menu) {
  add(menu, "Settings (iwmenu)", "network.settings-row", NULL);
}

void network_popover_menu_add_ping(GMenu *menu) {
  add(menu, "Ping", "network.ping-row", NULL);
}

void network_popover_menu_add_network(GMenu *menu, IO_NetworkData network) {
  char label[100];
  sprintf(label, "%s: %s", network.iface, network.address);
  GVariant *target_value = g_variant_new_string(network.address);
  add(menu, label, "network.network-row", target_value);
}

GMenu *network_popover_menu_new(void) {
  GMenu *menu = g_menu_new();
  network_popover_menu_add_settings(menu);
  network_popover_menu_add_ping(menu);
  return menu;
}
