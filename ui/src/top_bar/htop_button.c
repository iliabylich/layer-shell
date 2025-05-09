#include "ui/include/top_bar/htop_button.h"

struct _HtopButton {
  GtkButton parent_instance;
};

G_DEFINE_TYPE(HtopButton, htop_button, GTK_TYPE_BUTTON)

static void htop_button_class_init(HtopButtonClass *) {}

static void htop_button_init(HtopButton *self) {
  gtk_button_set_label(GTK_BUTTON(self), "HTop");
  gtk_widget_add_css_class(GTK_WIDGET(self), "widget");
  gtk_widget_add_css_class(GTK_WIDGET(self), "terminal");
  gtk_widget_add_css_class(GTK_WIDGET(self), "padded");
  gtk_widget_add_css_class(GTK_WIDGET(self), "clickable");
  gtk_widget_set_cursor(GTK_WIDGET(self),
                        gdk_cursor_new_from_name("pointer", NULL));
  gtk_widget_set_name(GTK_WIDGET(self), "HTop");
}

GtkWidget *htop_button_new() {
  return g_object_new(htop_button_get_type(), NULL);
}
