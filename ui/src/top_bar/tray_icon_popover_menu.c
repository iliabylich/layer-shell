#include "ui/include/top_bar/tray_icon_popover_menu.h"
#include "ui/include/utils/fmt.h"

static void visit(IO_TrayItem tray_item, GMenu *menu);

static void visit_all(IO_CArray_TrayItem items, GMenu *menu) {
  for (size_t idx = 0; idx < items.len; idx++) {
    IO_TrayItem child = items.ptr[idx];
    visit(child, menu);
  }
}

static void visit_regular(IO_TrayItem_IO_Regular_Body regular, GMenu *menu) {
  GMenuItem *menu_item = g_menu_item_new(regular.label, NULL);
  int_to_tray_action_name_prefixed(regular.id, action);
  g_menu_item_set_action_and_target_value(menu_item, action, NULL);

  g_menu_append_item(menu, menu_item);
  g_object_unref(menu_item);
}

static void visit_disabled(IO_TrayItem_IO_Disabled_Body disabled, GMenu *menu) {
  GMenuItem *menu_item = g_menu_item_new(disabled.label, NULL);
  g_menu_item_set_action_and_target_value(menu_item, "tray.noop", NULL);

  g_menu_append_item(menu, menu_item);
  g_object_unref(menu_item);
}

static void visit_checkbox(IO_TrayItem_IO_Checkbox_Body checkbox, GMenu *menu) {
  GMenuItem *menu_item = g_menu_item_new(checkbox.label, NULL);
  int_to_tray_action_name_prefixed(checkbox.id, action);
  g_menu_item_set_action_and_target_value(menu_item, action, NULL);

  g_menu_append_item(menu, menu_item);
  g_object_unref(menu_item);
}

static void visit_radio(IO_TrayItem_IO_Radio_Body radio, GMenu *menu) {
  GMenuItem *menu_item = g_menu_item_new(radio.label, NULL);
  int_to_tray_action_name_prefixed(radio.id, action);
  g_menu_item_set_action_and_target_value(menu_item, action,
                                          g_variant_new_boolean(true));

  g_menu_append_item(menu, menu_item);
  g_object_unref(menu_item);
}

static void visit_nested(IO_TrayItem_IO_Nested_Body nested, GMenu *menu) {
  GMenu *submenu = g_menu_new();
  visit_all(nested.children, submenu);

  GMenuItem *menu_item = g_menu_item_new(nested.label, NULL);
  g_menu_item_set_submenu(menu_item, G_MENU_MODEL(submenu));
  g_menu_append_item(menu, menu_item);
  g_object_unref(menu_item);
}

static void visit_section(IO_TrayItem_IO_Section_Body nested, GMenu *menu) {
  GMenu *section = g_menu_new();
  visit_all(nested.children, section);
  g_menu_append_section(menu, NULL, G_MENU_MODEL(section));
}

static void visit(IO_TrayItem tray_item, GMenu *menu) {
  switch (tray_item.tag) {
  case IO_TrayItem_Regular: {
    visit_regular(tray_item.regular, menu);
    return;
  }
  case IO_TrayItem_Disabled: {
    visit_disabled(tray_item.disabled, menu);
    return;
  }
  case IO_TrayItem_Checkbox: {
    visit_checkbox(tray_item.checkbox, menu);
    return;
  }
  case IO_TrayItem_Radio: {
    visit_radio(tray_item.radio, menu);
    return;
  }
  case IO_TrayItem_Nested: {
    visit_nested(tray_item.nested, menu);
    return;
  }
  case IO_TrayItem_Section: {
    visit_section(tray_item.section, menu);
    return;
  }
  }
}

GMenu *tray_icon_popover_menu_new(IO_CArray_TrayItem items) {
  GMenu *menu = g_menu_new();
  visit_all(items, menu);
  return menu;
}
