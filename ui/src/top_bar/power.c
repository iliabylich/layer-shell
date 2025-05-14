#include "ui/include/top_bar/power.h"
#include "gtk/gtk.h"
#include "ui/include/icons.h"

struct _Power {
  GtkButton parent_instance;
};

G_DEFINE_TYPE(Power, power, GTK_TYPE_BUTTON)

static void power_class_init(PowerClass *) {}

static const char *css_classes[] = {"widget", "power", "padded", "clickable",
                                    NULL};

static void power_init(Power *self) {
  gtk_widget_set_css_classes(GTK_WIDGET(self), css_classes);
  gtk_widget_set_cursor(GTK_WIDGET(self),
                        gdk_cursor_new_from_name("pointer", NULL));
  gtk_widget_set_name(GTK_WIDGET(self), "Power");

  GtkWidget *image = gtk_image_new_from_gicon(get_power_icon());
  gtk_button_set_child(GTK_BUTTON(self), image);
}

GtkWidget *power_new() { return g_object_new(power_get_type(), NULL); }
