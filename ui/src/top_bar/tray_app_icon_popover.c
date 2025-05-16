#include "ui/include/top_bar/tray_app_icon_popover.h"
#include "ui/include/top_bar/tray.h"
#include "ui/include/top_bar/tray_app_icon_popover_action_map.h"
#include "ui/include/top_bar/tray_app_icon_popover_menu.h"

GtkWidget *tray_app_icon_popover_new(IO_TrayItem tray_item, Tray *tray,
                                     GList **context_pool) {
  GSimpleActionGroup *action_group =
      tray_app_icon_popover_action_map_new(tray_item, tray, context_pool);
  GMenu *menu = tray_app_icon_popover_menu_new(tray_item);

  GtkWidget *self = gtk_popover_menu_new_from_model(G_MENU_MODEL(menu));
  gtk_popover_set_has_arrow(GTK_POPOVER(self), false);
  gtk_popover_menu_set_flags(GTK_POPOVER_MENU(self), GTK_POPOVER_MENU_NESTED);

  gtk_widget_insert_action_group(self, TRAY_ACTION_NAMESPACE,
                                 G_ACTION_GROUP(action_group));

  return self;
}
