#include "ui/include/top_bar/tray_menu_item.h"
#include "ui/include/top_bar/tray.h"
#include "ui/include/top_bar/tray_menu.h"

static char *action_name(size_t idx) {
  char *out = calloc(10, sizeof(char));
  sprintf(out, "%lu", idx);
  return out;
}
#define ACTION_NAME action_name(idx)

static char *namespaced_action_name(size_t idx) {
  char *out = calloc(10, sizeof(char));
  sprintf(out, "tray.%lu", idx);
  return out;
}
#define NAMESPACED_ACTION_NAME namespaced_action_name(idx)

typedef struct {
  Tray *tray;
  char *uuid;
} context_t;
char *strcopy(char *s) {
  size_t len = strlen(s);
  char *out = malloc(len + 1);
  strcpy(out, s);
  out[len] = 0;
  return out;
}
context_t *context_new(Tray *tray, char *uuid) {
  context_t *out = malloc(sizeof(context_t));
  out->tray = tray;
  out->uuid = strcopy(uuid);
  return out;
}
#define MAKE_CONTEXT context_new(tray, tray_item.uuid)

static void on_activate(GSimpleAction *, GVariant *, context_t *context) {
  tray_emit_triggered(context->tray, strcopy(context->uuid));
}

static GMenuItem *tray_nested_menu_item_new(IO_TrayItem tray_item,
                                            GActionGroup *action_group,
                                            Tray *tray) {
  GMenuItem *menu_item = g_menu_item_new(tray_item.label, NULL);
  GMenu *submenu = tray_menu_new(tray_item, action_group, tray);
  g_menu_item_set_submenu(menu_item, G_MENU_MODEL(submenu));
  return menu_item;
}

static GMenuItem *tray_checkbox_menu_item_new(IO_TrayItem tray_item,
                                              GActionGroup *action_group,
                                              size_t idx, Tray *tray) {
  GMenuItem *menu_item = g_menu_item_new(tray_item.label, NULL);
  GSimpleAction *action = g_simple_action_new_stateful(
      ACTION_NAME, NULL, g_variant_new_boolean(tray_item.toggle_state == 1));
  g_signal_connect(action, "activate", G_CALLBACK(on_activate), MAKE_CONTEXT);
  g_action_map_add_action(G_ACTION_MAP(action_group), G_ACTION(action));
  g_menu_item_set_action_and_target_value(menu_item, NAMESPACED_ACTION_NAME,
                                          NULL);
  return menu_item;
}

static GMenuItem *tray_radio_menu_item_new(IO_TrayItem tray_item,
                                           GActionGroup *action_group,
                                           size_t idx, Tray *tray) {
  GMenuItem *menu_item = g_menu_item_new(tray_item.label, NULL);
  GSimpleAction *action = g_simple_action_new_stateful(
      ACTION_NAME, G_VARIANT_TYPE_BOOLEAN,
      g_variant_new_boolean(tray_item.toggle_state == 1));
  g_signal_connect(action, "activate", G_CALLBACK(on_activate), MAKE_CONTEXT);
  g_action_map_add_action(G_ACTION_MAP(action_group), G_ACTION(action));
  g_menu_item_set_action_and_target_value(menu_item, NAMESPACED_ACTION_NAME,
                                          g_variant_new_boolean(true));
  return menu_item;
}

static GMenuItem *tray_regular_menu_item_new(IO_TrayItem tray_item,
                                             GActionGroup *action_group,
                                             size_t idx, Tray *tray) {
  GMenuItem *menu_item = g_menu_item_new(tray_item.label, NULL);
  GSimpleAction *action = g_simple_action_new(ACTION_NAME, NULL);
  g_signal_connect(action, "activate", G_CALLBACK(on_activate), MAKE_CONTEXT);
  g_action_map_add_action(G_ACTION_MAP(action_group), G_ACTION(action));
  g_menu_item_set_action_and_target_value(menu_item, NAMESPACED_ACTION_NAME,
                                          NULL);
  return menu_item;
}

static GMenuItem *tray_disabled_menu_item_new(IO_TrayItem tray_item) {
  GMenuItem *menu_item = g_menu_item_new(tray_item.label, NULL);
  g_menu_item_set_action_and_target_value(menu_item, "tray.noop", NULL);
  return menu_item;
}

GMenuItem *tray_menu_item_new(IO_TrayItem tray_item, GActionGroup *action_group,
                              size_t idx, Tray *tray) {
  if (strcmp(tray_item.children_display, "submenu") == 0) {
    return tray_nested_menu_item_new(tray_item, action_group, tray);
  } else if (tray_item.enabled) {
    if (strcmp(tray_item.toggle_type, "checkmark") == 0) {
      return tray_checkbox_menu_item_new(tray_item, action_group, idx, tray);
    } else if (strcmp(tray_item.toggle_type, "radio") == 0) {
      return tray_radio_menu_item_new(tray_item, action_group, idx, tray);
    } else {
      return tray_regular_menu_item_new(tray_item, action_group, idx, tray);
    }
  } else {
    return tray_disabled_menu_item_new(tray_item);
  }
}
