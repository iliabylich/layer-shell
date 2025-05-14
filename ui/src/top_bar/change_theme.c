#include "ui/include/top_bar/change_theme.h"
#include "ui/include/icons.h"

struct _ChangeTheme {
  GtkButton parent_instance;
};

G_DEFINE_TYPE(ChangeTheme, change_theme, GTK_TYPE_BUTTON)

static void change_theme_class_init(ChangeThemeClass *) {}

static const char *css_classes[] = {
    "widget", "change-theme", "padded", "clickable", NULL,
};

static void change_theme_init(ChangeTheme *self) {
  gtk_widget_set_css_classes(GTK_WIDGET(self), css_classes);
  gtk_widget_set_cursor(GTK_WIDGET(self),
                        gdk_cursor_new_from_name("pointer", NULL));
  gtk_widget_set_name(GTK_WIDGET(self), "ChangeTheme");

  GtkWidget *image = gtk_image_new_from_gicon(get_change_theme_icon());
  gtk_button_set_child(GTK_BUTTON(self), image);
}

GtkWidget *change_theme_new() { return g_object_new(CHANGE_THEME_TYPE, NULL); }
