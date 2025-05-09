#include "ui/include/top_bar/network_popover.h"
#include "ui/include/top_bar/network.h"

static void add_settings_row(GtkWidget *self) {
  GMenuItem *item = g_menu_item_new("Settings (nmtui)", "network.settings");
  GMenuModel *model = gtk_popover_menu_get_menu_model(GTK_POPOVER_MENU(self));
  g_menu_append_item(G_MENU(model), item);
}

static void add_ping_row(GtkWidget *self) {
  GMenuItem *item = g_menu_item_new("Ping", "network.ping");
  GMenuModel *model = gtk_popover_menu_get_menu_model(GTK_POPOVER_MENU(self));
  g_menu_append_item(G_MENU(model), item);
}

static void on_settings_row_clicked(GSimpleAction *, GVariant *,
                                    GtkWidget *parent) {
  network_emit_settings_clicked(NETWORK(parent));
}

static void on_ping_row_clicked(GSimpleAction *, GVariant *,
                                GtkWidget *parent) {
  network_emit_ping_clicked(NETWORK(parent));
}

static void on_network_row_clicked(GSimpleAction *, GVariant *parameter,
                                   GtkWidget *parent) {
  const char *address = g_variant_get_string(parameter, NULL);
  network_emit_network_clicked(NETWORK(parent), address);
}

GtkWidget *network_popover_new(Network *parent) {
  GMenu *menu = g_menu_new();
  GtkWidget *self = gtk_popover_menu_new_from_model(G_MENU_MODEL(menu));

  add_settings_row(self);
  add_ping_row(self);

  gtk_popover_set_has_arrow(GTK_POPOVER(self), false);

  GSimpleActionGroup *action_group = g_simple_action_group_new();
  GSimpleAction *action;

  action = g_simple_action_new("settings", NULL);
  g_signal_connect(action, "activate", G_CALLBACK(on_settings_row_clicked),
                   parent);
  g_action_map_add_action(G_ACTION_MAP(action_group), G_ACTION(action));

  action = g_simple_action_new("ping", NULL);
  g_signal_connect(action, "activate", G_CALLBACK(on_ping_row_clicked), parent);
  g_action_map_add_action(G_ACTION_MAP(action_group), G_ACTION(action));

  action = g_simple_action_new("network-row-clicked", G_VARIANT_TYPE_STRING);
  g_signal_connect(action, "activate", G_CALLBACK(on_network_row_clicked),
                   parent);
  g_action_map_add_action(G_ACTION_MAP(action_group), G_ACTION(action));

  gtk_widget_insert_action_group(GTK_WIDGET(self), "network",
                                 G_ACTION_GROUP(action_group));
  return self;
}

void network_popover_refresh(GtkWidget *self, IO_CArray_Network list) {
  GMenuModel *model = gtk_popover_menu_get_menu_model(GTK_POPOVER_MENU(self));
  g_menu_remove_all(G_MENU(model));
  for (size_t i = 0; i < list.len; i++) {
    IO_Network network = list.ptr[i];
    char format[100];
    sprintf(format, "%s: %s", network.iface, network.address);
    GMenuItem *item = g_menu_item_new(format, NULL);
    g_menu_item_set_action_and_target_value(
        item, "network.network-row-clicked",
        g_variant_new_string(network.address));
    g_menu_append_item(G_MENU(model), item);
  }
  add_settings_row(self);
  add_ping_row(self);
}
