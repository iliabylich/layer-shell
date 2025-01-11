#include "css.h"
#include <gtk/gtk.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static char *full_css();

static void on_css_parse_error(GtkCssProvider *, GtkCssSection *section,
                               GError *error, gpointer) {
  fprintf(stderr, "Failed to parse CSS: %s %s\n",
          gtk_css_section_to_string(section), error->message);
}

void load_css(void) {
  char *css = full_css();

  GtkCssProvider *provider = gtk_css_provider_new();
  g_signal_connect(provider, "parsing-error", G_CALLBACK(on_css_parse_error),
                   NULL);

  gtk_css_provider_load_from_string(provider, css);
  free(css);

  GdkDisplay *display = gdk_display_get_default();
  gtk_style_context_add_provider_for_display(
      display, GTK_STYLE_PROVIDER(provider),
      GTK_STYLE_PROVIDER_PRIORITY_APPLICATION);

  printf("Finished loading CSS...\n");
}

static char *read_file(const char *filepath) {
  char *buffer = 0;
  long length;
  FILE *f = fopen(filepath, "rb");

  if (f) {
    fseek(f, 0, SEEK_END);
    length = ftell(f);
    fseek(f, 0, SEEK_SET);
    buffer = malloc(length);
    if (buffer) {
      fread(buffer, 1, length, f);
    }
    fclose(f);
    return buffer;
  } else {
    exit(EXIT_FAILURE);
  }
}

const char MAIN_CSS[] = {
#embed "main.css" if_empty('-')
    , 0};
const size_t MAIN_CSS_LEN = sizeof(MAIN_CSS);

static char *full_css() {
  char theme_filepath[100];
  sprintf(theme_filepath, "%s/%s", getenv("HOME"), ".theme.css");
  char *theme = read_file(theme_filepath);
  size_t theme_len = strlen(theme);

  char *out = malloc(MAIN_CSS_LEN + theme_len + 1);
  memcpy(out, theme, theme_len);
  memcpy(out + theme_len, MAIN_CSS, MAIN_CSS_LEN);
  out[MAIN_CSS_LEN + theme_len] = 0;

  return out;
}
