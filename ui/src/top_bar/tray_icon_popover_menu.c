#include "ui/include/top_bar/tray_icon_popover_menu.h"
#include "ui/include/top_bar/tray_helper.h"
#include "ui/include/utils/fmt.h"

static GMenuItem *visit(IO_TrayItem tray_item);
static GMenuItem *visit_nested(IO_TrayItem tray_item);
static GMenuItem *visit_disabled(IO_TrayItem tray_item);
static GMenuItem *visit_checkbox(IO_TrayItem tray_item);
static GMenuItem *visit_radio(IO_TrayItem tray_item);
static GMenuItem *visit_regular(IO_TrayItem tray_item);
static GMenu *visit_nested_as_submenu(IO_TrayItem tray_item);

static GMenuItem *visit(IO_TrayItem tray_item) {
  if (TRAY_ITEM_IS_NESTED(tray_item)) {
    return visit_nested(tray_item);
  } else if (TRAY_ITEM_IS_DISABLED(tray_item)) {
    return visit_disabled(tray_item);
  } else if (TRAY_ITEM_IS_CHECKBOX(tray_item)) {
    return visit_checkbox(tray_item);
  } else if (TRAY_ITEM_IS_RADIO(tray_item)) {
    return visit_radio(tray_item);
  } else {
    return visit_regular(tray_item);
  }
}
static GMenuItem *visit_nested(IO_TrayItem tray_item) {
  GMenuItem *menu_item = g_menu_item_new(tray_item.label, NULL);
  GMenu *submenu = visit_nested_as_submenu(tray_item);
  g_menu_item_set_submenu(menu_item, G_MENU_MODEL(submenu));
  return menu_item;
}

static GMenuItem *visit_disabled(IO_TrayItem tray_item) {
  GMenuItem *menu_item = g_menu_item_new(tray_item.label, NULL);
  g_menu_item_set_action_and_target_value(menu_item, "tray.noop", NULL);
  return menu_item;
}

static GMenuItem *visit_checkbox(IO_TrayItem tray_item) {
  GMenuItem *menu_item = g_menu_item_new(tray_item.label, NULL);
  int_to_tray_action_name_prefixed(tray_item.id, action);
  g_menu_item_set_action_and_target_value(menu_item, action, NULL);
  return menu_item;
}

static GMenuItem *visit_radio(IO_TrayItem tray_item) {
  GMenuItem *menu_item = g_menu_item_new(tray_item.label, NULL);
  int_to_tray_action_name_prefixed(tray_item.id, action);
  g_menu_item_set_action_and_target_value(menu_item, action,
                                          g_variant_new_boolean(true));
  return menu_item;
}

static GMenuItem *visit_regular(IO_TrayItem tray_item) {
  GMenuItem *menu_item = g_menu_item_new(tray_item.label, NULL);
  int_to_tray_action_name_prefixed(tray_item.id, action);
  g_menu_item_set_action_and_target_value(menu_item, action, NULL);
  return menu_item;
}

static GMenu *visit_nested_as_submenu(IO_TrayItem tray_item) {
  GMenu *menu = g_menu_new();

  for (size_t idx = 0; idx < tray_item.children.len; idx++) {
    IO_TrayItem child = tray_item.children.ptr[idx];
    if (!child.visible) {
      continue;
    }

    GMenuItem *menu_item = visit(child);
    g_menu_append_item(menu, menu_item);
    g_object_unref(menu_item);
  }

  return menu;
}

GMenu *tray_icon_popover_menu_new(IO_TrayItem tray_item) {
  return visit_nested_as_submenu(tray_item);
}
