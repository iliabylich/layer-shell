#include "ui/include/top_bar/language.h"
#include "gtk/gtk.h"

static const char *css_classes[] = {"widget", "language", "padded", NULL};

GtkWidget *language_new() {
  GtkWidget *label = gtk_label_new("--");

  gtk_widget_set_css_classes(label, css_classes);
  gtk_widget_set_name(label, "Language");

  return label;
}

void language_refresh(Language *self, const char *lang) {
  const char *text;

  if (strcmp(lang, "English (US)") == 0) {
    text = "EN";
  } else if (strcmp(lang, "Polish") == 0) {
    text = "PL";
  } else {
    text = "??";
  }

  gtk_label_set_text(self, text);
}
