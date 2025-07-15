#pragma once

#include "bindings.h"
#include "ui/base_window.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(CapsLockWindow, caps_lock_window, CAPS_LOCK_WINDOW, WINDOW,
                     BaseWindow)

#define CAPS_LOCK_WINDOW(obj)                                                  \
  G_TYPE_CHECK_INSTANCE_CAST(obj, caps_lock_window_get_type(), CapsLockWindow)

GtkWidget *caps_lock_window_new(GtkApplication *app);

void caps_lock_window_refresh(CapsLockWindow *caps_lock_window,
                              IO_ControlCapsLockToggledEvent event);
