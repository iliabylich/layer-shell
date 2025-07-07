#include "ui/network_popover.h"
#include "ui/logger.h"

LOGGER("NetworkPopover", 2)

enum {
  SIGNAL_CLICKED_SETTINGS = 0,
  SIGNAL_CLICKED_PING,
  SIGNAL_CLICKED_ADDRESS,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

struct _NetworkPopover {
  GtkWidget parent_instance;

  GtkWidget *root;
  GMenu *menu;
  GSimpleActionGroup *action_group;
};

G_DEFINE_TYPE(NetworkPopover, network_popover, GTK_TYPE_WIDGET)

static void on_settings_clicked(GSimpleAction *, GVariant *,
                                NetworkPopover *self) {
  g_signal_emit(self, signals[SIGNAL_CLICKED_SETTINGS], 0);
}

static void on_ping_clicked(GSimpleAction *, GVariant *, NetworkPopover *self) {
  g_signal_emit(self, signals[SIGNAL_CLICKED_PING], 0);
}

static void on_address_clicked(GSimpleAction *, GVariant *parameter,
                               NetworkPopover *self) {
  const char *address = g_variant_get_string(parameter, NULL);
  g_signal_emit(self, signals[SIGNAL_CLICKED_ADDRESS], 0, address);
}

static void add_action(NetworkPopover *self, const char *action_name,
                       GCallback callback, const GVariantType *parameter_type) {
  GSimpleAction *action = g_simple_action_new(action_name, parameter_type);
  g_signal_connect(action, "activate", callback, self);
  g_action_map_add_action(G_ACTION_MAP(self->action_group), G_ACTION(action));
}

static void append_row(NetworkPopover *self, const char *text,
                       const char *action_name, GVariant *target_value) {
  GMenuItem *item = g_menu_item_new(text, NULL);
  g_menu_item_set_action_and_target_value(item, action_name, target_value);
  g_menu_append_item(self->menu, item);
  g_object_unref(item);
}

static void network_popover_init(NetworkPopover *self) {
  LOG("init");

  self->menu = g_menu_new();
  self->action_group = g_simple_action_group_new();

  add_action(self, "settings", G_CALLBACK(on_settings_clicked), NULL);
  add_action(self, "ping", G_CALLBACK(on_ping_clicked), NULL);
  add_action(self, "address", G_CALLBACK(on_address_clicked),
             G_VARIANT_TYPE_STRING);

  append_row(self, "Settings (iwmenu)", "network.settings", NULL);
  append_row(self, "Ping", "network.ping", NULL);

  self->root = gtk_popover_menu_new_from_model(G_MENU_MODEL(self->menu));
  gtk_popover_set_has_arrow(GTK_POPOVER(self->root), false);

  gtk_widget_insert_action_group(GTK_WIDGET(self), "network",
                                 G_ACTION_GROUP(self->action_group));
  gtk_widget_set_parent(self->root, GTK_WIDGET(self));
}

static void network_popover_dispose(GObject *object) {
  LOG("dispose");
  NetworkPopover *self = NETWORK_POPOVER(object);
  g_clear_pointer(&self->root, gtk_widget_unparent);
  G_OBJECT_CLASS(network_popover_parent_class)->dispose(object);
}

static void network_popover_class_init(NetworkPopoverClass *klass) {
  LOG("class init");
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = network_popover_dispose;

  signals[SIGNAL_CLICKED_SETTINGS] = g_signal_new_class_handler(
      "clicked-settings", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST,
      NULL, NULL, NULL, NULL, G_TYPE_NONE, 0);
  signals[SIGNAL_CLICKED_PING] = g_signal_new_class_handler(
      "clicked-ping", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST,
      NULL, NULL, NULL, NULL, G_TYPE_NONE, 0);
  signals[SIGNAL_CLICKED_ADDRESS] = g_signal_new_class_handler(
      "clicked-address", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST,
      NULL, NULL, NULL, NULL, G_TYPE_NONE, 1, G_TYPE_STRING);

  gtk_widget_class_set_layout_manager_type(GTK_WIDGET_CLASS(klass),
                                           GTK_TYPE_BOX_LAYOUT);
}

GtkWidget *network_popover_new() {
  return g_object_new(network_popover_get_type(), NULL);
}

void network_popover_open(NetworkPopover *self) {
  gtk_popover_popup(GTK_POPOVER(self->root));
}

void network_popover_refresh(NetworkPopover *self, IO_NetworkListEvent event) {
  while (g_menu_model_get_n_items(G_MENU_MODEL(self->menu)) != 2) {
    g_menu_remove(self->menu, 2);
  }

  for (size_t i = 0; i < event.list.len; i++) {
    IO_NetworkData network = event.list.ptr[i];
    char label[100];
    sprintf(label, "%s: %s", network.iface, network.address);
    GVariant *target_value = g_variant_new_string(network.address);
    append_row(self, label, "network.address", target_value);
  }
}
