#include "ui/session_overlay.h"
#include "ui/gobject_helper.h"
#include "ui/logger.h"

LOGGER("SessionOverlay", 0)

enum {
  SIGNAL_CLICKED_LOCK = 0,
  SIGNAL_CLICKED_REBOOT,
  SIGNAL_CLICKED_SHUTDOWN,
  SIGNAL_CLICKED_LOGOUT,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

struct _SessionOverlay {
  GtkWidget parent_instance;
};

G_DEFINE_TYPE(SessionOverlay, session_overlay, BASE_OVERLAY_TYPE)

static void set_visible(SessionOverlay *self, gboolean visible) {
  gobject_set_nested(G_OBJECT(base_overlay_get_model(BASE_OVERLAY(self))),
                     "overlays", "session", visible);
}

static void lock_clicked(SessionOverlay *self) {
  g_signal_emit(self, signals[SIGNAL_CLICKED_LOCK], 0);
  set_visible(self, false);
}

static void reboot_clicked(SessionOverlay *self) {
  g_signal_emit(self, signals[SIGNAL_CLICKED_REBOOT], 0);
  set_visible(self, false);
}

static void shutdown_clicked(SessionOverlay *self) {
  g_signal_emit(self, signals[SIGNAL_CLICKED_SHUTDOWN], 0);
  set_visible(self, false);
}

static void logout_clicked(SessionOverlay *self) {
  g_signal_emit(self, signals[SIGNAL_CLICKED_LOGOUT], 0);
  set_visible(self, false);
}

static void toggle_requested(BaseOverlay *, gpointer data) {
  SessionOverlay *self = SESSION_OVERLAY(data);
  gobject_toggle_nested(G_OBJECT(base_overlay_get_model(BASE_OVERLAY(self))),
                        "overlays", "session");
}

static void session_overlay_init(SessionOverlay *self) {
  LOG("init");
  gtk_widget_init_template(GTK_WIDGET(self));
  g_signal_connect(self, "toggle-requested", G_CALLBACK(toggle_requested),
                   self);
}

static void session_overlay_class_init(SessionOverlayClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);

  signals[SIGNAL_CLICKED_LOCK] = g_signal_new_class_handler(
      "clicked-lock", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST,
      NULL, NULL, NULL, NULL, G_TYPE_NONE, 0);
  signals[SIGNAL_CLICKED_REBOOT] = g_signal_new_class_handler(
      "clicked-reboot", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST,
      NULL, NULL, NULL, NULL, G_TYPE_NONE, 0);
  signals[SIGNAL_CLICKED_SHUTDOWN] = g_signal_new_class_handler(
      "clicked-shutdown", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST,
      NULL, NULL, NULL, NULL, G_TYPE_NONE, 0);
  signals[SIGNAL_CLICKED_LOGOUT] = g_signal_new_class_handler(
      "clicked-logout", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST,
      NULL, NULL, NULL, NULL, G_TYPE_NONE, 0);

  GtkWidgetClass *widget_class = GTK_WIDGET_CLASS(klass);
  gtk_widget_class_set_template_from_resource(
      widget_class, "/layer-shell/session_overlay.ui");
  gtk_widget_class_bind_template_callback(widget_class, lock_clicked);
  gtk_widget_class_bind_template_callback(widget_class, reboot_clicked);
  gtk_widget_class_bind_template_callback(widget_class, shutdown_clicked);
  gtk_widget_class_bind_template_callback(widget_class, logout_clicked);
}

GtkWidget *session_overlay_new(GtkApplication *app, IOModel *model) {
  return g_object_new(session_overlay_get_type(), "application", app, "model",
                      model, NULL);
}
