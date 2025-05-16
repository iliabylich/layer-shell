#include "ui/include/top_bar/tray_app_icon_popover_action_map.h"
#include "bindings.h"
#include "ui/include/macros.h"
#include "ui/include/top_bar/tray.h"
#include "ui/include/top_bar/tray_app_icon_context.h"

typedef tray_app_icon_context_t context_t;
#define context_new_root tray_app_icon_context_new_root
#define context_new_child tray_app_icon_context_new_child

static void on_activate(GSimpleAction *, GVariant *, context_t *context) {
  tray_emit_triggered(context->tray, context->uuid);
}

static void visit(IO_TrayItem tray_item, context_t *context);
static void visit_nested(IO_TrayItem tray_item, context_t *context);
static void visit_disabled(IO_TrayItem tray_item, context_t *context);
static void visit_checkbox(IO_TrayItem tray_item, context_t *context);
static void visit_radio(IO_TrayItem tray_item, context_t *context);
static void visit_regular(IO_TrayItem tray_item, context_t *context);

static void visit(IO_TrayItem tray_item, context_t *context) {
  if (TRAY_ITEM_IS_NESTED(tray_item)) {
    visit_nested(tray_item, context);
  } else if (TRAY_ITEM_IS_DISABLED(tray_item)) {
    visit_disabled(tray_item, context);
  } else if (TRAY_ITEM_IS_CHECKBOX(tray_item)) {
    visit_checkbox(tray_item, context);
  } else if (TRAY_ITEM_IS_RADIO(tray_item)) {
    visit_radio(tray_item, context);
  } else {
    visit_regular(tray_item, context);
  }
}

static void visit_nested(IO_TrayItem tray_item, context_t *context) {
  for (size_t child_idx = 0; child_idx < tray_item.children.len; child_idx++) {
    IO_TrayItem child = tray_item.children.ptr[child_idx];
    if (!child.visible) {
      continue;
    }

    context_t *child_context =
        context_new_child(context, child_idx, child.uuid);

    visit(child, child_context);
  }
}

static void visit_disabled(IO_TrayItem, context_t *) {}

static void visit_checkbox(IO_TrayItem tray_item, context_t *context) {
  GSimpleAction *action = g_simple_action_new_stateful(
      context->name, NULL, g_variant_new_boolean(tray_item.toggle_state == 1));
  g_signal_connect(action, "activate", G_CALLBACK(on_activate), context);
  g_action_map_add_action(G_ACTION_MAP(context->action_group),
                          G_ACTION(action));
}

static void visit_radio(IO_TrayItem tray_item, context_t *context) {
  GSimpleAction *action = g_simple_action_new_stateful(
      context->name, G_VARIANT_TYPE_BOOLEAN,
      g_variant_new_boolean(tray_item.toggle_state == 1));
  g_signal_connect(action, "activate", G_CALLBACK(on_activate), context);
  g_action_map_add_action(G_ACTION_MAP(context->action_group),
                          G_ACTION(action));
}

static void visit_regular(IO_TrayItem, context_t *context) {
  GSimpleAction *action = g_simple_action_new(context->name, NULL);
  g_signal_connect(action, "activate", G_CALLBACK(on_activate), context);
  g_action_map_add_action(G_ACTION_MAP(context->action_group),
                          G_ACTION(action));
}

GActionGroup *tray_app_icon_popover_action_map_new(IO_TrayItem tray_item,
                                                   Tray *tray, GList **pool) {
  GActionGroup *action_group = G_ACTION_GROUP(g_simple_action_group_new());

  context_t *context = context_new_root(tray, action_group, pool);

  visit_nested(tray_item, context);

  return action_group;
}
