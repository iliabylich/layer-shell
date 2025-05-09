#include "ui/include/top_bar/tray_menu.h"
#include "ui/include/top_bar/tray.h"
#include "ui/include/top_bar/tray_menu_item.h"

GMenu *tray_menu_new(IO_TrayItem tray_item, GActionGroup *action_group,
                     Tray *tray) {
  GMenu *menu = g_menu_new();
  for (size_t idx = 0; idx < tray_item.children.len; idx++) {
    IO_TrayItem child = tray_item.children.ptr[idx];
    if (!child.visible) {
      continue;
    }

    GMenuItem *menu_item = tray_menu_item_new(child, action_group, idx, tray);
    g_menu_append_item(menu, menu_item);
  }
  return menu;
}
