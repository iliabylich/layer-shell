#include "ui/clock.h"
#include "ui/logger.h"

LOGGER("Clock", 1)

struct _Clock {
  GtkWidget parent_instance;

  GtkWidget *root;
};

G_DEFINE_TYPE(Clock, clock, GTK_TYPE_WIDGET)

static void clock_init(Clock *self) {
  LOG("init");

  self->root = gtk_label_new("--");
  gtk_widget_add_css_class(self->root, "widget");
  gtk_widget_add_css_class(self->root, "clock");
  gtk_widget_add_css_class(self->root, "padded");

  gtk_widget_set_parent(self->root, GTK_WIDGET(self));
}

static void clock_dispose(GObject *object) {
  LOG("dispose");

  Clock *self = CLOCK(object);
  g_clear_pointer(&self->root, gtk_widget_unparent);
  G_OBJECT_CLASS(clock_parent_class)->dispose(object);
}

static void clock_class_init(ClockClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = clock_dispose;
  gtk_widget_class_set_layout_manager_type(GTK_WIDGET_CLASS(klass),
                                           GTK_TYPE_BOX_LAYOUT);
}

GtkWidget *clock_new(void) { return g_object_new(clock_get_type(), NULL); }

void clock_refresh(Clock *self, IO_CString time) {
  gtk_label_set_text(GTK_LABEL(self->root), time);
}
