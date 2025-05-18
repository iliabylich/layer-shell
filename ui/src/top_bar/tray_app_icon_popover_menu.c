#include "ui/include/top_bar/tray_app_icon_popover_menu.h"
#include "ui/include/top_bar/tray.h"
#include "ui/include/top_bar/tray_helper.h"

static GMenuItem *visit(IO_TrayItem tray_item, const char *name);
static GMenuItem *visit_nested(IO_TrayItem tray_item, const char *name);
static GMenuItem *visit_disabled(IO_TrayItem tray_item, const char *name);
static GMenuItem *visit_checkbox(IO_TrayItem tray_item, const char *name);
static GMenuItem *visit_radio(IO_TrayItem tray_item, const char *name);
static GMenuItem *visit_regular(IO_TrayItem tray_item, const char *name);
static GMenu *visit_nested_as_submenu(IO_TrayItem tray_item, const char *name);

static GMenuItem *visit(IO_TrayItem tray_item, const char *name) {
  if (TRAY_ITEM_IS_NESTED(tray_item)) {
    return visit_nested(tray_item, name);
  } else if (TRAY_ITEM_IS_DISABLED(tray_item)) {
    return visit_disabled(tray_item, name);
  } else if (TRAY_ITEM_IS_CHECKBOX(tray_item)) {
    return visit_checkbox(tray_item, name);
  } else if (TRAY_ITEM_IS_RADIO(tray_item)) {
    return visit_radio(tray_item, name);
  } else {
    return visit_regular(tray_item, name);
  }
}
static GMenuItem *visit_nested(IO_TrayItem tray_item, const char *name) {
  GMenuItem *menu_item = g_menu_item_new(tray_item.label, NULL);
  GMenu *submenu = visit_nested_as_submenu(tray_item, name);
  g_menu_item_set_submenu(menu_item, G_MENU_MODEL(submenu));
  return menu_item;
}

static GMenuItem *visit_disabled(IO_TrayItem tray_item, const char *) {
  GMenuItem *menu_item = g_menu_item_new(tray_item.label, NULL);
  g_menu_item_set_action_and_target_value(menu_item, "tray.noop", NULL);
  return menu_item;
}

static GMenuItem *visit_checkbox(IO_TrayItem tray_item, const char *name) {
  GMenuItem *menu_item = g_menu_item_new(tray_item.label, NULL);
  g_menu_item_set_action_and_target_value(menu_item, name, NULL);
  return menu_item;
}

static GMenuItem *visit_radio(IO_TrayItem tray_item, const char *name) {
  GMenuItem *menu_item = g_menu_item_new(tray_item.label, NULL);
  g_menu_item_set_action_and_target_value(menu_item, name,
                                          g_variant_new_boolean(true));
  return menu_item;
}

static GMenuItem *visit_regular(IO_TrayItem tray_item, const char *name) {
  GMenuItem *menu_item = g_menu_item_new(tray_item.label, NULL);
  g_menu_item_set_action_and_target_value(menu_item, name, NULL);
  return menu_item;
}

static GMenu *visit_nested_as_submenu(IO_TrayItem tray_item, const char *name) {
  GMenu *menu = g_menu_new();

  for (size_t idx = 0; idx < tray_item.children.len; idx++) {
    IO_TrayItem child = tray_item.children.ptr[idx];
    if (!child.visible) {
      continue;
    }

    char child_name[100];
    sprintf(child_name, "%s:%lu", name, idx);
    GMenuItem *menu_item = visit(child, child_name);
    g_menu_append_item(menu, menu_item);
  }

  return menu;
}

GMenu *tray_app_icon_popover_menu_new(IO_TrayItem tray_item) {
  const char *root_name = TRAY_ACTION_NAMESPACE "." TRAY_ACTION_ROOT_PREFIX;
  return visit_nested_as_submenu(tray_item, root_name);
}
