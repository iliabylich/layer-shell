#include "ui/include/top_bar/tray_app_icon_context.h"

static char *strcopy(const char *s) {
  size_t len = strlen(s);
  char *out = malloc(len + 1);
  strcpy(out, s);
  out[len] = 0;
  return out;
}

static char *strconcat(const char *prefix, size_t suffix) {
  char buffer[100];
  sprintf(buffer, "%s:%lu", prefix, suffix);
  return strcopy(buffer);
}

tray_app_icon_context_t *
tray_app_icon_context_new_root(Tray *tray, GActionGroup *action_group,
                               GList **pool) {
  tray_app_icon_context_t *context = calloc(1, sizeof(tray_app_icon_context_t));
  context->tray = tray;
  context->action_group = action_group;
  context->pool = pool;
  context->name = strcopy(TRAY_ACTION_ROOT_PREFIX);

  *pool = g_list_append(*pool, context);

  return context;
}

tray_app_icon_context_t *
tray_app_icon_context_new_child(tray_app_icon_context_t *parent,
                                size_t child_idx, const char *uuid) {
  tray_app_icon_context_t *context = malloc(sizeof(tray_app_icon_context_t));
  context->tray = parent->tray;
  context->action_group = parent->action_group;
  context->pool = parent->pool;
  context->name = strconcat(parent->name, child_idx);
  context->uuid = strcopy(uuid);

  *context->pool = g_list_append(*context->pool, context);

  return context;
}

void tray_app_icon_context_free(tray_app_icon_context_t *context) {
  free(context->name);
  if (context->uuid) {
    free(context->uuid);
  }
  free(context);
}
