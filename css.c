#include "css.h"
#include "bindings.h"
#include <gtk/gtk.h>
#include <stdio.h>

static void on_css_parse_error(__attribute__((unused)) GtkCssProvider *self,
                               GtkCssSection *section, GError *error,
                               __attribute__((unused)) gpointer user_data) {
  fprintf(stderr, "Failed to parse CSS: %s %s\n",
          gtk_css_section_to_string(section), error->message);
}

void load_css(void) {
  GtkCssProvider *provider = gtk_css_provider_new();
  g_signal_connect(provider, "parsing-error", G_CALLBACK(on_css_parse_error),
                   NULL);

  LAYER_SHELL_IO_CString css = layer_shell_io_main_css();
  gtk_css_provider_load_from_string(provider, css.ptr);
  free(css.ptr);

  GdkDisplay *display = gdk_display_get_default();
  gtk_style_context_add_provider_for_display(
      display, GTK_STYLE_PROVIDER(provider),
      GTK_STYLE_PROVIDER_PRIORITY_APPLICATION);

  printf("Finished loading CSS...\n");
}
