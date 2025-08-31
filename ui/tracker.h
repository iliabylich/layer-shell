#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Tracker, tracker, TRACKER, WIDGET, GtkWidget)

#define TRACKER(obj)                                                           \
  G_TYPE_CHECK_INSTANCE_CAST(obj, tracker_get_type(), Tracker)

GtkWidget *tracker_new(void);
void tracker_refresh(Tracker *tracker, IO_TrackerUpdatedEvent event);
