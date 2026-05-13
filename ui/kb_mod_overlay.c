#include "ui/kb_mod_overlay.h"
#include "ui/logger.h"

LOGGER("KbModOverlay", 0)

struct _KbModOverlay {
  GtkWidget parent_instance;
};

G_DEFINE_TYPE(KbModOverlay, kb_mod_overlay, BASE_OVERLAY_TYPE)

static char *format_kb_mod_icon(GObject *, bool enabled) {
  return g_strdup(enabled ? "" : "");
}

static char *format_kb_mod_label(GObject *, guint kind, bool enabled) {
  const char *name = "CapsLock";
  if (kind == IO_KbModKind_NumLock) {
    name = "NumLock";
  }
  return g_strdup_printf("%s %s", name, enabled ? "ON" : "OFF");
}

static void kb_mod_overlay_init(KbModOverlay *self) {
  LOG("init");
  gtk_widget_init_template(GTK_WIDGET(self));
}

static void kb_mod_overlay_class_init(KbModOverlayClass *klass) {
  LOG("class init");

  GtkWidgetClass *widget_class = GTK_WIDGET_CLASS(klass);
  gtk_widget_class_set_template_from_resource(widget_class,
                                              "/layer-shell/kb_mod_overlay.ui");
  gtk_widget_class_bind_template_callback(widget_class, format_kb_mod_icon);
  gtk_widget_class_bind_template_callback(widget_class, format_kb_mod_label);
}

GtkWidget *kb_mod_overlay_new(GtkApplication *app, IOModel *model) {
  return g_object_new(kb_mod_overlay_get_type(), "application", app, "model",
                      model, NULL);
}
