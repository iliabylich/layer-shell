#include "ui/css.h"
#include "main.scss.xxd"
#include <assert.h>
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
  size_t len;

  memset(theme_css_path, 0, sizeof(theme_css_path));
  const char *home = getenv("HOME");
  assert(home != NULL);
  len = sprintf(theme_css_path, "%s/.config/layer-shell/theme.css", home);
  assert(len > 0);
  fprintf(stderr, "Theme css path: %s\n", theme_css_path);

  memset(theme_css, 0, sizeof(theme_css));
  assert(theme_css != NULL);
  FILE *f = fopen(theme_css_path, "rb");
  if (f == NULL) {
    fprintf(stderr, "Failed to load theme CSS from %s\n", theme_css_path);
    fclose(f);
    return;
  }
  fseek(f, 0, SEEK_END);
  len = ftell(f);
  assert(len > 0);
  if (len > sizeof(theme_css)) {
    fprintf(stderr, "not enough space for theme.css: %lu cs %lu\n", len,
            sizeof(theme_css));
    assert(false);
  }
  fprintf(stderr, "Have main.css, len is %lu\n", len);
  fseek(f, 0, SEEK_SET);
  fread(theme_css, 1, len, f);
  fclose(f);
}

static char full_css[20000] = {0};

static void merge_css() {
  size_t len = snprintf(full_css, sizeof(full_css), "%.*s%.*s",
                        (unsigned int)strlen(theme_css), theme_css,
                        main_scss_len, main_scss);
  assert(len > 0);
  if (len > sizeof(full_css)) {
    fprintf(stderr, "not enough space for full CSS: %lu cs %lu\n", len,
            sizeof(full_css));
    assert(false);
  }
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
