#include "ui/session_window_model.h"

struct _SessionWindowModel {
  WindowModel parent_instance;
};

G_DEFINE_TYPE(SessionWindowModel, session_window_model, window_model_get_type())

static void session_window_model_init(SessionWindowModel *) {}
static void session_window_model_class_init(SessionWindowModelClass *) {}

SessionWindowModel *session_window_model_new(void) {
  return g_object_new(session_window_model_get_type(), NULL);
}
