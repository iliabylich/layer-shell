#include "ui/include/top_bar/language.h"
#include "ui/include/builder.h"

GtkWidget *language_init() { return top_bar_get_widget("LANGUAGE"); }

void language_refresh(GtkWidget *self, const char *lang) {
  const char *text;

  if (strcmp(lang, "English (US)") == 0) {
    text = "EN";
  } else if (strcmp(lang, "Polish") == 0) {
    text = "PL";
  } else {
    text = "??";
  }

  gtk_label_set_text(GTK_LABEL(self), text);
}
