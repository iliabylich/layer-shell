#include "ui/include/top_bar/change_theme.h"
#include "ui/include/icons.h"

struct _ChangeTheme {
  GtkButton parent_instance;
};

G_DEFINE_TYPE(ChangeTheme, change_theme, GTK_TYPE_BUTTON)

static void change_theme_class_init(ChangeThemeClass *) {}

static void change_theme_init(ChangeTheme *) {}

GtkWidget *change_theme_new() {
  return g_object_new(CHANGE_THEME_TYPE,
                      //
                      "css-classes",
                      (const char *[]){
                          "widget",
                          "change-theme",
                          "padded",
                          "clickable",
                          NULL,
                      },
                      //
                      "cursor", gdk_cursor_new_from_name("pointer", NULL),
                      //
                      "name", "ChangeTheme",
                      //
                      "child",
                      gtk_image_new_from_gicon(get_change_theme_icon()),
                      //
                      NULL);
}
