#include "ui/include/top_bar/clock.h"

struct _Clock {
  GtkBox parent_instance;

  GtkWidget *label;
};

G_DEFINE_TYPE(Clock, clock, GTK_TYPE_BOX)

static void clock_class_init(ClockClass *) {}

static void clock_init(Clock *self) {
  self->label = gtk_label_new("--");
  gtk_widget_add_css_class(GTK_WIDGET(self->label), "widget");
  gtk_widget_add_css_class(GTK_WIDGET(self->label), "clock");
  gtk_widget_add_css_class(GTK_WIDGET(self->label), "padded");
  gtk_widget_set_name(GTK_WIDGET(self), "Clock");

  gtk_box_append(GTK_BOX(self), self->label);
}

GtkWidget *clock_new() { return g_object_new(clock_get_type(), NULL); }

void clock_refresh(Clock *self, IO_CString time) {
  gtk_label_set_text(GTK_LABEL(self->label), time);
}
