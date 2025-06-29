#include "ui/include/top_bar/tray_icon_popover_action_map.h"
#include "bindings.h"
#include "ui/include/top_bar/tray.h"
#include "ui/include/top_bar/tray_helper.h"
#include "ui/include/utils/fmt.h"
#include "ui/include/utils/strclone.h"

#define UUID_KEY "uuid"
static void set_uuid(GSimpleAction *action, const char *uuid) {
  g_object_set_data_full(G_OBJECT(action), UUID_KEY, strclone(uuid), free);
}
static const char *get_uuid(GSimpleAction *action) {
  return g_object_get_data(G_OBJECT(action), UUID_KEY);
}

#define CALLBACK_KEY "callback"
static void set_cb(GSimpleAction *action, tray_triggered_f cb) {
  g_object_set_data(G_OBJECT(action), CALLBACK_KEY, (gpointer)(size_t)cb);
}
static tray_triggered_f get_cb(GSimpleAction *action) {
  return (tray_triggered_f)(size_t)g_object_get_data(G_OBJECT(action),
                                                     CALLBACK_KEY);
}

static void on_activate(GSimpleAction *action, GVariant *) {
  tray_triggered_f cb = get_cb(action);
  const char *uuid = get_uuid(action);
  cb(uuid);
}

static void visit(IO_TrayItem tray_item, GActionGroup *action_group,
                  tray_triggered_f cb);
static void visit_nested(IO_TrayItem tray_item, GActionGroup *action_group,
                         tray_triggered_f cb);
static void visit_disabled(IO_TrayItem tray_item, GActionGroup *action_group,
                           tray_triggered_f cb);
static void visit_checkbox(IO_TrayItem tray_item, GActionGroup *action_group,
                           tray_triggered_f cb);
static void visit_radio(IO_TrayItem tray_item, GActionGroup *action_group,
                        tray_triggered_f cb);
static void visit_regular(IO_TrayItem tray_item, GActionGroup *action_group,
                          tray_triggered_f cb);

static void visit(IO_TrayItem tray_item, GActionGroup *action_group,
                  tray_triggered_f cb) {
  if (TRAY_ITEM_IS_NESTED(tray_item)) {
    visit_nested(tray_item, action_group, cb);
  } else if (TRAY_ITEM_IS_DISABLED(tray_item)) {
    visit_disabled(tray_item, action_group, cb);
  } else if (TRAY_ITEM_IS_CHECKBOX(tray_item)) {
    visit_checkbox(tray_item, action_group, cb);
  } else if (TRAY_ITEM_IS_RADIO(tray_item)) {
    visit_radio(tray_item, action_group, cb);
  } else {
    visit_regular(tray_item, action_group, cb);
  }
}

static void visit_nested(IO_TrayItem tray_item, GActionGroup *action_group,
                         tray_triggered_f cb) {
  for (size_t child_idx = 0; child_idx < tray_item.children.len; child_idx++) {
    IO_TrayItem child = tray_item.children.ptr[child_idx];
    if (!child.visible) {
      continue;
    }

    visit(child, action_group, cb);
  }
}

static void visit_disabled(IO_TrayItem, GActionGroup *, tray_triggered_f) {}

static void visit_checkbox(IO_TrayItem tray_item, GActionGroup *action_group,
                           tray_triggered_f cb) {
  int_to_tray_action_name_no_prefix(tray_item.id, action_name);
  GVariant *state = g_variant_new_boolean(tray_item.toggle_state == 1);
  GSimpleAction *action =
      g_simple_action_new_stateful(action_name, NULL, state);
  set_uuid(action, tray_item.uuid);
  set_cb(action, cb);
  g_signal_connect(action, "activate", G_CALLBACK(on_activate), NULL);
  g_action_map_add_action(G_ACTION_MAP(action_group), G_ACTION(action));
}

static void visit_radio(IO_TrayItem tray_item, GActionGroup *action_group,
                        tray_triggered_f cb) {
  int_to_tray_action_name_no_prefix(tray_item.id, action_name);
  GVariant *state = g_variant_new_boolean(tray_item.toggle_state == 1);
  GSimpleAction *action =
      g_simple_action_new_stateful(action_name, G_VARIANT_TYPE_BOOLEAN, state);
  set_uuid(action, tray_item.uuid);
  set_cb(action, cb);
  g_signal_connect(action, "activate", G_CALLBACK(on_activate), NULL);
  g_action_map_add_action(G_ACTION_MAP(action_group), G_ACTION(action));
}

static void visit_regular(IO_TrayItem tray_item, GActionGroup *action_group,
                          tray_triggered_f cb) {
  int_to_tray_action_name_no_prefix(tray_item.id, action_name);
  GSimpleAction *action = g_simple_action_new(action_name, NULL);
  set_uuid(action, tray_item.uuid);
  set_cb(action, cb);
  g_signal_connect(action, "activate", G_CALLBACK(on_activate), NULL);
  g_action_map_add_action(G_ACTION_MAP(action_group), G_ACTION(action));
}

GActionGroup *tray_icon_popover_action_map_new(IO_TrayItem tray_item,
                                               tray_triggered_f cb) {
  GActionGroup *action_group = G_ACTION_GROUP(g_simple_action_group_new());

  visit_nested(tray_item, action_group, cb);

  return action_group;
}
