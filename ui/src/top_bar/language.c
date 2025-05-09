#include "ui/include/top_bar/language.h"

struct _Language {
  GtkBox parent_instance;

  GtkWidget *label;
};

G_DEFINE_TYPE(Language, language, GTK_TYPE_BOX)

static void language_class_init(LanguageClass *) {}

static void language_init(Language *self) {
  gtk_widget_add_css_class(GTK_WIDGET(self), "widget");
  gtk_widget_add_css_class(GTK_WIDGET(self), "language");
  gtk_widget_add_css_class(GTK_WIDGET(self), "padded");
  gtk_widget_set_name(GTK_WIDGET(self), "Language");

  self->label = gtk_label_new("--");
  gtk_box_append(GTK_BOX(self), self->label);
}

GtkWidget *language_new() { return g_object_new(language_get_type(), NULL); }

void language_refresh(Language *self, IO_CString lang) {
  const char *label;

  if (strcmp(lang, "English (US)") == 0) {
    label = "EN";
  } else if (strcmp(lang, "Polish") == 0) {
    label = "PL";
  } else {
    label = "??";
  }

  gtk_label_set_text(GTK_LABEL(self->label), label);
}
