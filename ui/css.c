#include "ui/css.h"
#include "main.scss.xxd"
#include "ui/assertions.h"
#include <gtk/gtk.h>
#include <stdio.h>

static void css_parse_error(GtkCssProvider *, GtkCssSection *section,
                            GError *error, gpointer) {
  fprintf(stderr, "Failed to parse CSS: %s %s\n",
          gtk_css_section_to_string(section), error->message);
}

static GtkCssProvider *provider = NULL;

void css_load(void) {
  provider = gtk_css_provider_new();
  g_signal_connect(provider, "parsing-error", G_CALLBACK(css_parse_error),
                   NULL);

  gtk_css_provider_load_from_bytes(
      provider, g_bytes_new_static(main_scss, main_scss_len));

  GdkDisplay *display = gdk_display_get_default();
  gtk_style_context_add_provider_for_display(
      display, GTK_STYLE_PROVIDER(provider),
      GTK_STYLE_PROVIDER_PRIORITY_APPLICATION);

  fprintf(stderr, "Finished loading CSS...\n");
}
