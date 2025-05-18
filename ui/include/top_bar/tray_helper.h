#define TRAY_ITEM_IS_NESTED(tray_item)                                         \
  strcmp(tray_item.children_display, "submenu") == 0
#define TRAY_ITEM_IS_DISABLED(tray_item) !tray_item.enabled
#define TRAY_ITEM_IS_CHECKBOX(tray_item)                                       \
  strcmp(tray_item.toggle_type, "checkmark") == 0
#define TRAY_ITEM_IS_RADIO(tray_item)                                          \
  strcmp(tray_item.toggle_type, "radio") == 0
