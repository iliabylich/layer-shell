#pragma once

#include "ui/tracker_task.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(TrackerRow, tracker_row, TRACKER_ROW, WIDGET, GtkWidget)

#define TRACKER_ROW(obj)                                                       \
  G_TYPE_CHECK_INSTANCE_CAST(obj, tracker_row_get_type(), TrackerRow)

GtkWidget *tracker_row_new();
void tracker_row_update(TrackerRow *tracker_row, TrackerTask *task);
