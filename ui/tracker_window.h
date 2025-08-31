#pragma once

#include "bindings.h"
#include "ui/base_window.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(TrackerWindow, tracker_window, TRACKER_WINDOW, WINDOW,
                     BaseWindow)

#define TRACKER_WINDOW(obj)                                                    \
  G_TYPE_CHECK_INSTANCE_CAST(obj, tracker_window_get_type(), TrackerWindow)

GtkWidget *tracker_window_new(GtkApplication *app);
void tracker_window_toggle(TrackerWindow *tracker_window);
void tracker_window_refresh(TrackerWindow *tracker_window,
                            IO_TrackerUpdatedEvent event);
