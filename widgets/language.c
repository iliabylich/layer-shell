#include "language.h"
#include "../bindings.h"
#include <gtk/gtk.h>

#define _(name) language_widget_ns_##name

static GtkWidget *_(widget);
static GtkWidget *_(label);

static GtkWidget *_(init)(void) {
  _(widget) = gtk_center_box_new();
  gtk_widget_add_css_class(_(widget), "widget");
  gtk_widget_add_css_class(_(widget), "language");
  gtk_widget_add_css_class(_(widget), "padded");
  gtk_widget_set_name(_(widget), "Language");

  _(label) = gtk_label_new("--");
  gtk_center_box_set_center_widget(GTK_CENTER_BOX(_(widget)), _(label));

  return _(widget);
}

static void _(on_io_event)(const LAYER_SHELL_IO_Event *event) {
  switch (event->tag) {
  case Language: {
    if (strcmp(event->language.lang, "English (US)") == 0) {
      gtk_label_set_label(GTK_LABEL(_(label)), "EN");
    } else if (strcmp(event->language.lang, "Polish") == 0) {
      gtk_label_set_label(GTK_LABEL(_(label)), "PL");
    } else {
      gtk_label_set_label(GTK_LABEL(_(label)), "??");
    }
    break;
  }
  default: {
    break;
  }
  }
}

static void _(activate)(void) { layer_shell_io_subscribe(_(on_io_event)); }

widget_t LANGUAGE_WIDGET = {.init = _(init), .activate = _(activate)};
