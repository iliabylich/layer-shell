#include "ui/include/top_bar/change_theme.h"
#include "ui/include/icons.h"

struct _ChangeTheme {
  GtkButton parent_instance;
};

G_DEFINE_TYPE(ChangeTheme, change_theme, GTK_TYPE_BUTTON)

static void change_theme_class_init(ChangeThemeClass *) {}

static void change_theme_init(ChangeTheme *self) {
  gtk_widget_add_css_class(GTK_WIDGET(self), "widget");
  gtk_widget_add_css_class(GTK_WIDGET(self), "change-theme");
  gtk_widget_add_css_class(GTK_WIDGET(self), "padded");
  gtk_widget_add_css_class(GTK_WIDGET(self), "clickable");
  gtk_widget_set_cursor(GTK_WIDGET(self),
                        gdk_cursor_new_from_name("pointer", NULL));
  gtk_widget_set_name(GTK_WIDGET(self), "ChangeTheme");

  GtkWidget *image = gtk_image_new_from_gicon(get_change_theme_icon());
  gtk_button_set_child(GTK_BUTTON(self), image);
}

GtkWidget *change_theme_new() {
  return g_object_new(change_theme_get_type(), NULL);
}
