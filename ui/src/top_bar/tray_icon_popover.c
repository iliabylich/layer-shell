#include "ui/include/top_bar/tray_icon_popover.h"
#include "ui/include/top_bar/tray.h"
#include "ui/include/top_bar/tray_icon_popover_action_map.h"
#include "ui/include/top_bar/tray_icon_popover_menu.h"

GtkWidget *tray_icon_popover_new(IO_CArray_TrayItem items,
                                 tray_triggered_f cb) {
  GActionGroup *action_group = tray_icon_popover_action_map_new(items, cb);
  GMenu *menu = tray_icon_popover_menu_new(items);

  GtkWidget *self = gtk_popover_menu_new_from_model(G_MENU_MODEL(menu));
  gtk_popover_set_has_arrow(GTK_POPOVER(self), false);
  gtk_popover_menu_set_flags(GTK_POPOVER_MENU(self), GTK_POPOVER_MENU_NESTED);

  gtk_widget_insert_action_group(self, "tray", action_group);

  return self;
}
