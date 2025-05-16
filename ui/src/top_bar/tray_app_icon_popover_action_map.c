#include "ui/include/top_bar/tray_app_icon_popover_action_map.h"
#include "bindings.h"
#include "gio/gio.h"
#include "ui/include/macros.h"
#include "ui/include/top_bar/tray.h"

typedef struct {
  char *uuid;
  char *name;
  Tray *tray;
} context_t;

static char *strcopy(const char *s) {
  size_t len = strlen(s);
  char *out = malloc(len + 1);
  strcpy(out, s);
  out[len] = 0;
  return out;
}

static char *strconcat(const char *prefix, size_t idx) {
  char buffer[100];
  sprintf(buffer, "%s:%lu", prefix, idx);
  return strcopy(buffer);
}

static context_t *context_new(Tray *tray, const char *name, const char *uuid) {
  context_t *context = malloc(sizeof(context_t));
  context->tray = tray;
  context->name = strcopy(name);
  context->uuid = strcopy(uuid);
  return context;
}

static void on_activate(GSimpleAction *, GVariant *, context_t *context) {
  tray_emit_triggered(context->tray, context->uuid);
}

static void visit(IO_TrayItem tray_item, GActionGroup *action_group,
                  context_t *context);
static void visit_nested(IO_TrayItem tray_item, GActionGroup *action_group,
                         context_t *context);
static void visit_disabled(IO_TrayItem tray_item, GActionGroup *action_group,
                           context_t *context);
static void visit_checkbox(IO_TrayItem tray_item, GActionGroup *action_group,
                           context_t *context);
static void visit_radio(IO_TrayItem tray_item, GActionGroup *action_group,
                        context_t *context);
static void visit_regular(IO_TrayItem tray_item, GActionGroup *action_group,
                          context_t *context);

static void visit(IO_TrayItem tray_item, GActionGroup *action_group,
                  context_t *context) {
  if (TRAY_ITEM_IS_NESTED(tray_item)) {
    visit_nested(tray_item, action_group, context);
  } else if (TRAY_ITEM_IS_DISABLED(tray_item)) {
    visit_disabled(tray_item, action_group, context);
  } else if (TRAY_ITEM_IS_CHECKBOX(tray_item)) {
    visit_checkbox(tray_item, action_group, context);
  } else if (TRAY_ITEM_IS_RADIO(tray_item)) {
    visit_radio(tray_item, action_group, context);
  } else {
    visit_regular(tray_item, action_group, context);
  }
}

static void visit_nested(IO_TrayItem tray_item, GActionGroup *action_group,
                         context_t *context) {
  for (size_t idx = 0; idx < tray_item.children.len; idx++) {
    IO_TrayItem child = tray_item.children.ptr[idx];
    if (!child.visible) {
      continue;
    }

    context_t *child_context =
        context_new(context->tray, strconcat(context->name, idx), child.uuid);
    visit(child, action_group, child_context);
  }

  free(context->uuid);
  free(context);
}

static void visit_disabled(IO_TrayItem, GActionGroup *, context_t *context) {
  free(context->uuid);
  free(context);
}

static void visit_checkbox(IO_TrayItem tray_item, GActionGroup *action_group,
                           context_t *context) {
  GSimpleAction *action = g_simple_action_new_stateful(
      context->name, NULL, g_variant_new_boolean(tray_item.toggle_state == 1));
  g_signal_connect(action, "activate", G_CALLBACK(on_activate), context);
  g_action_map_add_action(G_ACTION_MAP(action_group), G_ACTION(action));
}

static void visit_radio(IO_TrayItem tray_item, GActionGroup *action_group,
                        context_t *context) {
  GSimpleAction *action = g_simple_action_new_stateful(
      context->name, G_VARIANT_TYPE_BOOLEAN,
      g_variant_new_boolean(tray_item.toggle_state == 1));
  g_signal_connect(action, "activate", G_CALLBACK(on_activate), context);
  g_action_map_add_action(G_ACTION_MAP(action_group), G_ACTION(action));
}

static void visit_regular(IO_TrayItem, GActionGroup *action_group,
                          context_t *context) {
  GSimpleAction *action = g_simple_action_new(context->name, NULL);
  g_signal_connect(action, "activate", G_CALLBACK(on_activate), context);
  g_action_map_add_action(G_ACTION_MAP(action_group), G_ACTION(action));
}

GSimpleActionGroup *tray_app_icon_popover_action_map_new(IO_TrayItem tray_item,
                                                         Tray *tray) {
  GSimpleActionGroup *action_group = g_simple_action_group_new();

  context_t *context = context_new(tray, TRAY_ACTION_ROOT_PREFIX, "root");

  visit_nested(tray_item, G_ACTION_GROUP(action_group), context);

  return action_group;
}
