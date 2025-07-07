#include "ui/change_theme.h"
#include "ui/logger.h"

LOGGER("ChangeTheme", 1)

enum {
  SIGNAL_CLICKED = 0,
  N_SIGNALS,
};
static guint signals[N_SIGNALS] = {0};

struct _ChangeTheme {
  GtkWidget parent_instance;

  GtkWidget *root;
};

G_DEFINE_TYPE(ChangeTheme, change_theme, GTK_TYPE_WIDGET)

static void on_click(GtkWidget *, ChangeTheme *self) {
  g_signal_emit(self, signals[SIGNAL_CLICKED], 0);
}

static void change_theme_init(ChangeTheme *self) {
  LOG("init");

  self->root = gtk_button_new_with_label("");
  gtk_widget_add_css_class(self->root, "widget");
  gtk_widget_add_css_class(self->root, "change-theme");
  gtk_widget_add_css_class(self->root, "padded");
  gtk_widget_add_css_class(self->root, "clickable");
  gtk_widget_set_cursor_from_name(self->root, "pointer");
  g_signal_connect(self->root, "clicked", G_CALLBACK(on_click), self);

  gtk_widget_set_parent(self->root, GTK_WIDGET(self));
}

static void change_theme_dispose(GObject *object) {
  LOG("dispose");
  ChangeTheme *self = CHANGE_THEME(object);

  g_clear_pointer(&self->root, gtk_widget_unparent);

  G_OBJECT_CLASS(change_theme_parent_class)->dispose(object);
}

static void change_theme_class_init(ChangeThemeClass *klass) {
  LOG("class init");
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = change_theme_dispose;

  signals[SIGNAL_CLICKED] = g_signal_new_class_handler(
      "clicked", G_OBJECT_CLASS_TYPE(object_class), G_SIGNAL_RUN_LAST, NULL,
      NULL, NULL, NULL, G_TYPE_NONE, 0);

  gtk_widget_class_set_layout_manager_type(GTK_WIDGET_CLASS(klass),
                                           GTK_TYPE_BOX_LAYOUT);
}

GtkWidget *change_theme_new(void) {
  return g_object_new(change_theme_get_type(), NULL);
}
