#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(TrackerTask, tracker_task, TRACKER_TASK, OBJECT, GObject)

#define TRACKER_TASK_TYPE tracker_task_get_type()
#define TRACKER_TASK(obj)                                                      \
  G_TYPE_CHECK_INSTANCE_CAST(obj, TRACKER_TASK_TYPE, TrackerTask)

TrackerTask *tracker_task_new(const char *title, const char *uuid,
                              const char *duration, bool selected);

TrackerTask **tracker_task_batch_new(IO_CArray_Task data);

const char *tracker_task_get_title(TrackerTask *task);
const char *tracker_task_get_uuid(TrackerTask *task);
const char *tracker_task_get_duration(TrackerTask *task);
bool tracker_task_get_selected(TrackerTask *task);
