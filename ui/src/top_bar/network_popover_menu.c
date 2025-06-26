#include "ui/include/top_bar/network_popover_menu.h"
#include "ui/include/top_bar/network.h"

static void add(GMenu *menu, const char *label, const char *action,
                GVariant *target_value) {
  GMenuItem *item = g_menu_item_new(label, NULL);
  g_menu_item_set_action_and_target_value(item, action, target_value);
  g_menu_append_item(menu, item);
}

void network_popover_menu_add_settings(GMenu *menu) {
  add(menu, NETWORK_SETTINGS_LABEL, NETWORK_SETTINGS_DETAILED_ACTION, NULL);
}

void network_popover_menu_add_ping(GMenu *menu) {
  add(menu, NETWORK_PING_LABEL, NETWORK_PING_DETAILED_ACTION, NULL);
}

void network_popover_menu_add_network(GMenu *menu, IO_NetworkData network) {
  char label[100];
  sprintf(label, "%s: %s", network.iface, network.address);
  GVariant *target_value = g_variant_new_string(network.address);
  add(menu, label, NETWORK_ROW_DETAILED_ACTION, target_value);
}

GMenu *network_popover_menu_new(void) {
  GMenu *menu = g_menu_new();
  network_popover_menu_add_settings(menu);
  network_popover_menu_add_ping(menu);
  return menu;
}
