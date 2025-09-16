#include "ui/css.h"
#include "main.scss.xxd"
#include "ui/assertions.h"
#include <gtk/gtk.h>
#include <stdio.h>

static void on_css_parse_error(GtkCssProvider *, GtkCssSection *section,
                               GError *error, gpointer) {
  fprintf(stderr, "Failed to parse CSS: %s %s\n",
          gtk_css_section_to_string(section), error->message);
}

static char theme_css_path[100] = {0};
static char theme_css[5000] = {0};

static void read_theme_css() {
  const char *home = getenv("HOME");
  assert(home, "failed to get $HOME");

  memset(theme_css_path, 0, sizeof(theme_css_path));
  checked_fmt(theme_css_path, "%s/.config/layer-shell/theme.css", home);
  fprintf(stderr, "Theme css path: %s\n", theme_css_path);

  memset(theme_css, 0, sizeof(theme_css));
  FILE *f = fopen(theme_css_path, "rb");
  assert(f != NULL, "failed to open theme css file");
  fseek(f, 0, SEEK_END);
  size_t len = ftell(f);
  assert(len < sizeof(theme_css), "not enough space for theme.css: %lu vs %lu",
         len, sizeof(theme_css));
  fseek(f, 0, SEEK_SET);
  size_t read = fread(theme_css, 1, len, f);
  assert(read == len, "failed to fully read theme.css");
  fclose(f);
  fprintf(stderr, "Have main.css, len is %lu\n", len);
}

static char full_css[20000] = {0};

static void merge_css() {
  checked_fmt(full_css, "%.*s%.*s", (unsigned int)strlen(theme_css), theme_css,
              main_scss_len, main_scss);
  fprintf(stderr, "Have full CSS, len is %lu\n", strlen(full_css));
}

static GtkCssProvider *provider = NULL;

void css_load(void) {
  read_theme_css();
  merge_css();

  provider = gtk_css_provider_new();
  g_signal_connect(provider, "parsing-error", G_CALLBACK(on_css_parse_error),
                   NULL);

  gtk_css_provider_load_from_string(provider, full_css);

  GdkDisplay *display = gdk_display_get_default();
  gtk_style_context_add_provider_for_display(
      display, GTK_STYLE_PROVIDER(provider),
      GTK_STYLE_PROVIDER_PRIORITY_APPLICATION);

  fprintf(stderr, "Finished loading CSS...\n");
}

void css_reload(void) {
  fprintf(stderr, "Reloading styles...\n");
  GdkDisplay *display = gdk_display_get_default();
  gtk_style_context_remove_provider_for_display(display,
                                                GTK_STYLE_PROVIDER(provider));
  css_load();
}
