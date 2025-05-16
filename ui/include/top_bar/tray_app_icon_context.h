#pragma once

#include "ui/include/top_bar/tray.h"

typedef struct {
  char *uuid;
  char *name;
  Tray *tray;
} tray_app_icon_context_t;

tray_app_icon_context_t *
tray_app_icon_context_new_root(Tray *tray, const char *name, const char *uuid);

tray_app_icon_context_t *
tray_app_icon_context_new_child(tray_app_icon_context_t *parent,
                                size_t child_idx, const char *uuid);

void tray_app_icon_context_free(tray_app_icon_context_t *context);
