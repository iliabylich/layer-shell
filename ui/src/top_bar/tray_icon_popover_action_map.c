#include "ui/include/top_bar/tray_icon_popover_action_map.h"
#include "ui/include/top_bar/tray.h"
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

static void visit(IO_TrayItem item, GActionGroup *action_group,
                  tray_triggered_f cb);

static void visit_all(IO_CArray_TrayItem items, GActionGroup *action_group,
                      tray_triggered_f cb) {
  for (size_t i = 0; i < items.len; i++) {
    IO_TrayItem child = items.ptr[i];
    visit(child, action_group, cb);
  }
}

static void visit_regular(IO_TrayItem_IO_Regular_Body regular,
                          GActionGroup *action_group, tray_triggered_f cb) {
  int_to_tray_action_name_no_prefix(regular.id, action_name);
  GSimpleAction *action = g_simple_action_new(action_name, NULL);
  set_uuid(action, regular.uuid);
  set_cb(action, cb);
  g_signal_connect(action, "activate", G_CALLBACK(on_activate), NULL);
  g_action_map_add_action(G_ACTION_MAP(action_group), G_ACTION(action));
}

static void visit_disabled(IO_TrayItem_IO_Disabled_Body, GActionGroup *,
                           tray_triggered_f) {}

static void visit_checkbox(IO_TrayItem_IO_Checkbox_Body checkbox,
                           GActionGroup *action_group, tray_triggered_f cb) {
  int_to_tray_action_name_no_prefix(checkbox.id, action_name);
  GVariant *state = g_variant_new_boolean(checkbox.checked);
  GSimpleAction *action =
      g_simple_action_new_stateful(action_name, NULL, state);
  set_uuid(action, checkbox.uuid);
  set_cb(action, cb);
  g_signal_connect(action, "activate", G_CALLBACK(on_activate), NULL);
  g_action_map_add_action(G_ACTION_MAP(action_group), G_ACTION(action));
}

static void visit_radio(IO_TrayItem_IO_Radio_Body radio,
                        GActionGroup *action_group, tray_triggered_f cb) {
  int_to_tray_action_name_no_prefix(radio.id, action_name);
  GVariant *state = g_variant_new_boolean(radio.selected);
  GSimpleAction *action =
      g_simple_action_new_stateful(action_name, G_VARIANT_TYPE_BOOLEAN, state);
  set_uuid(action, radio.uuid);
  set_cb(action, cb);
  g_signal_connect(action, "activate", G_CALLBACK(on_activate), NULL);
  g_action_map_add_action(G_ACTION_MAP(action_group), G_ACTION(action));
}

static void visit_nested(IO_TrayItem_IO_Nested_Body nested,
                         GActionGroup *action_group, tray_triggered_f cb) {
  visit_all(nested.children, action_group, cb);
}

static void visit_section(IO_TrayItem_IO_Section_Body section,
                          GActionGroup *action_group, tray_triggered_f cb) {
  visit_all(section.children, action_group, cb);
}

static void visit(IO_TrayItem tray_item, GActionGroup *action_group,
                  tray_triggered_f cb) {
  switch (tray_item.tag) {
  case IO_TrayItem_Regular: {
    visit_regular(tray_item.regular, action_group, cb);
    return;
  }
  case IO_TrayItem_Disabled: {
    visit_disabled(tray_item.disabled, action_group, cb);
    return;
  }
  case IO_TrayItem_Checkbox: {
    visit_checkbox(tray_item.checkbox, action_group, cb);
    return;
  }
  case IO_TrayItem_Radio: {
    visit_radio(tray_item.radio, action_group, cb);
    return;
  }
  case IO_TrayItem_Nested: {
    visit_nested(tray_item.nested, action_group, cb);
    return;
  }
  case IO_TrayItem_Section: {
    visit_section(tray_item.section, action_group, cb);
    return;
  }
  }
}

GActionGroup *tray_icon_popover_action_map_new(IO_CArray_TrayItem items,
                                               tray_triggered_f cb) {
  GActionGroup *action_group = G_ACTION_GROUP(g_simple_action_group_new());

  visit_all(items, action_group, cb);

  return action_group;
}
