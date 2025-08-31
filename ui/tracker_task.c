#include "ui/tracker_task.h"

struct _TrackerTask {
  GObject parent_instance;
  char *title;
  char *uuid;
  char *duration;
  bool selected;
};

G_DEFINE_TYPE(TrackerTask, tracker_task, g_object_get_type())

static void tracker_task_init(TrackerTask *) {}

static void tracker_task_dispose(GObject *object) {
  TrackerTask *self = TRACKER_TASK(object);
  g_clear_pointer(&self->title, g_free);
  g_clear_pointer(&self->uuid, g_free);
  g_clear_pointer(&self->duration, g_free);
  G_OBJECT_CLASS(tracker_task_parent_class)->dispose(object);
}

static void tracker_task_class_init(TrackerTaskClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = tracker_task_dispose;
}

TrackerTask *tracker_task_new(const char *title, const char *uuid,
                              const char *duration, bool selected) {
  TrackerTask *self = g_object_new(tracker_task_get_type(), NULL);
  self->title = g_strdup(title);
  self->uuid = g_strdup(uuid);
  self->duration = g_strdup(duration);
  self->selected = selected;
  return self;
}

const char *tracker_task_get_title(TrackerTask *self) { return self->title; }
const char *tracker_task_get_uuid(TrackerTask *self) { return self->uuid; }
const char *tracker_task_get_duration(TrackerTask *self) {
  return self->duration;
}
bool tracker_task_get_selected(TrackerTask *task) { return task->selected; }
