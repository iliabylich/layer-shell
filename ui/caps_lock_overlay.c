#include "ui/caps_lock_overlay.h"
#include "ui/logger.h"

LOGGER("CapsLockOverlay", 0)

struct _CapsLockOverlay {
  GtkWidget parent_instance;
};

G_DEFINE_TYPE(CapsLockOverlay, caps_lock_overlay, BASE_OVERLAY_TYPE)

static char *format_caps_icon(GObject *, bool enabled) {
  return g_strdup(enabled ? "" : "");
}

static char *format_caps_label(GObject *, bool enabled) {
  return g_strdup(enabled ? "CapsLock ON" : "CapsLock OFF");
}

static void caps_lock_overlay_init(CapsLockOverlay *self) {
  LOG("init");
  gtk_widget_init_template(GTK_WIDGET(self));
}

static void caps_lock_overlay_class_init(CapsLockOverlayClass *klass) {
  LOG("class init");

  GtkWidgetClass *widget_class = GTK_WIDGET_CLASS(klass);
  gtk_widget_class_set_template_from_resource(
      widget_class, "/layer-shell/caps_lock_overlay.ui");
  gtk_widget_class_bind_template_callback(widget_class, format_caps_icon);
  gtk_widget_class_bind_template_callback(widget_class, format_caps_label);
}

GtkWidget *caps_lock_overlay_new(GtkApplication *app, IOModel *model) {
  return g_object_new(caps_lock_overlay_get_type(), "application", app, "model",
                      model, NULL);
}
