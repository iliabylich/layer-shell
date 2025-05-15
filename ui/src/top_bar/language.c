#include "ui/include/top_bar/language.h"
#include "ui/include/macros.h"

GtkWidget *language_new() {
  // clang-format off
  return g_object_new(
      GTK_TYPE_LABEL,
      "label", "--",
      "css-classes", CSS("widget", "language", "padded"),
      "name", "Language",
      NULL);
  // clang-format on
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
