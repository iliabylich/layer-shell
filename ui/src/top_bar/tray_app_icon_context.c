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
tray_app_icon_context_new_root(Tray *tray, const char *name, const char *uuid) {
  tray_app_icon_context_t *context = malloc(sizeof(tray_app_icon_context_t));
  context->tray = tray;
  context->name = strcopy(name);
  context->uuid = strcopy(uuid);
  return context;
}

tray_app_icon_context_t *
tray_app_icon_context_new_child(tray_app_icon_context_t *parent,
                                size_t child_idx, const char *uuid) {
  tray_app_icon_context_t *context = malloc(sizeof(tray_app_icon_context_t));
  context->tray = parent->tray;
  context->name = strconcat(parent->name, child_idx);
  context->uuid = strcopy(uuid);
  return context;
}

void tray_app_icon_context_free(tray_app_icon_context_t *context) {
  free(context->name);
  if (context->uuid) {
    free(context->uuid);
  }
  free(context);
}
