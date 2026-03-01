#pragma once

#include "ui/base_window.h"
#include "ui/io_model.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(CapsLockWindow, caps_lock_window, CAPS_LOCK_WINDOW, WINDOW,
                     BaseWindow)

#define CAPS_LOCK_WINDOW(obj)                                                  \
  G_TYPE_CHECK_INSTANCE_CAST(obj, caps_lock_window_get_type(), CapsLockWindow)

GtkWidget *caps_lock_window_new(GtkApplication *app, IOModel *model);
