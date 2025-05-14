#include "ui/include/top_bar/tray_app.h"
#include "gtk/gtk.h"
#include "ui/include/top_bar/tray_app_icon.h"
#include "ui/include/top_bar/tray_menu.h"

static void on_click(GtkGestureClick *, gint, gdouble, gdouble,
                     GtkWidget *popover_menu) {
  gtk_popover_popup(GTK_POPOVER(popover_menu));
}

GtkWidget *tray_app_new(IO_TrayApp tray_app, Tray *tray) {
  GtkWidget *icon = tray_app_icon_new(tray_app.icon);

  GSimpleActionGroup *action_group = g_simple_action_group_new();
  GMenu *menu =
      tray_menu_new(tray_app.root_item, G_ACTION_GROUP(action_group), tray);

  GtkWidget *popover_menu = gtk_popover_menu_new_from_model(G_MENU_MODEL(menu));
  gtk_popover_set_has_arrow(GTK_POPOVER(popover_menu), false);
  gtk_popover_menu_set_flags(GTK_POPOVER_MENU(popover_menu),
                             GTK_POPOVER_MENU_NESTED);
  gtk_widget_set_parent(popover_menu, icon);

  GtkGesture *gesture = gtk_gesture_click_new();
  g_signal_connect(gesture, "pressed", G_CALLBACK(on_click), popover_menu);
  gtk_widget_add_controller(icon, GTK_EVENT_CONTROLLER(gesture));
  gtk_widget_insert_action_group(icon, "tray", G_ACTION_GROUP(action_group));

  return icon;
}
