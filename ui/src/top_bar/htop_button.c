#include "ui/include/top_bar/htop_button.h"
#include "gtk/gtk.h"

struct _HtopButton {
  GtkButton parent_instance;
};

G_DEFINE_TYPE(HtopButton, htop_button, GTK_TYPE_BUTTON)

static void htop_button_class_init(HtopButtonClass *) {}

static const char *css_classes[] = {"widget", "terminal", "padded", "clickable",
                                    NULL};

static void htop_button_init(HtopButton *self) {
  gtk_button_set_label(GTK_BUTTON(self), "HTop");
  gtk_widget_set_css_classes(GTK_WIDGET(self), css_classes);
  gtk_widget_set_cursor(GTK_WIDGET(self),
                        gdk_cursor_new_from_name("pointer", NULL));
  gtk_widget_set_name(GTK_WIDGET(self), "HTop");
}

GtkWidget *htop_button_new() { return g_object_new(HTOP_BUTTON_TYPE, NULL); }
