#include "ui/include/css.h"
#include "main.scss.xxd"
#include "ui/include/buffer.h"
#include <gtk/gtk.h>

static void on_css_parse_error(GtkCssProvider *, GtkCssSection *section,
                               GError *error, gpointer) {
  fprintf(stderr, "Failed to parse CSS: %s %s\n",
          gtk_css_section_to_string(section), error->message);
}

static buffer_t theme_css(void) {
  char theme_filepath[100];
  sprintf(theme_filepath, "%s/.config/layer-shell/theme.css", getenv("HOME"));
  return buffer_from_file(theme_filepath);
}

static buffer_t main_css(void) {
  return buffer_from_const_string((const char *)main_scss, main_scss_len);
}

GtkCssProvider *provider = NULL;

void css_load(void) {
  buffer_t css = buffer_merge(theme_css(), main_css());

  provider = gtk_css_provider_new();
  g_signal_connect(provider, "parsing-error", G_CALLBACK(on_css_parse_error),
                   NULL);

  gtk_css_provider_load_from_string(provider, css.ptr);
  buffer_free(css);

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
