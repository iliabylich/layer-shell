#include "ui/include/top_bar/network_popover.h"
#include "ui/include/top_bar/network.h"
#include "ui/include/top_bar/network_popover_action_map.h"
#include "ui/include/top_bar/network_popover_menu.h"

GtkWidget *network_popover_new(GtkWidget *parent) {
  GMenu *menu = network_popover_menu_new();

  GSimpleActionGroup *action_group =
      network_popover_action_map_new(GTK_WIDGET(parent));

  GtkWidget *self = gtk_popover_menu_new_from_model(G_MENU_MODEL(menu));
  gtk_popover_set_has_arrow(GTK_POPOVER(self), false);

  gtk_widget_insert_action_group(GTK_WIDGET(self), NETWORK_NAMESPACE,
                                 G_ACTION_GROUP(action_group));
  return self;
}

void network_popover_refresh(GtkWidget *self, IO_CArray_NetworkData list) {
  GMenuModel *menu = gtk_popover_menu_get_menu_model(GTK_POPOVER_MENU(self));
  g_menu_remove_all(G_MENU(menu));
  for (size_t i = 0; i < list.len; i++) {
    network_popover_menu_add_network(G_MENU(menu), list.ptr[i]);
  }
  network_popover_menu_add_settings(G_MENU(menu));
  network_popover_menu_add_ping(G_MENU(menu));
}
