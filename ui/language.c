#include "ui/language.h"
#include "ui/logger.h"

LOGGER("Language", 1)

struct _Language {
  GtkWidget parent_instance;

  GtkWidget *root;
};

G_DEFINE_TYPE(Language, language, GTK_TYPE_WIDGET)

static void language_init(Language *self) {
  LOG("init");

  self->root = gtk_label_new("--");
  gtk_widget_add_css_class(self->root, "widget");
  gtk_widget_add_css_class(self->root, "language");
  gtk_widget_add_css_class(self->root, "padded");

  gtk_widget_set_parent(self->root, GTK_WIDGET(self));
}

static void language_dispose(GObject *object) {
  LOG("dispose");

  Language *self = LANGUAGE(object);
  g_clear_pointer(&self->root, gtk_widget_unparent);
  G_OBJECT_CLASS(language_parent_class)->dispose(object);
}

static void language_class_init(LanguageClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = language_dispose;
  gtk_widget_class_set_layout_manager_type(GTK_WIDGET_CLASS(klass),
                                           GTK_TYPE_BOX_LAYOUT);
}

GtkWidget *language_new(void) {
  return g_object_new(language_get_type(), NULL);
}

void language_refresh(Language *self, IO_LanguageEvent event) {
  const char *text;

  if (strcmp(event.lang, "English (US)") == 0) {
    text = "EN";
  } else if (strcmp(event.lang, "Polish") == 0) {
    text = "PL";
  } else {
    text = "??";
  }

  gtk_label_set_text(GTK_LABEL(self->root), text);
}
