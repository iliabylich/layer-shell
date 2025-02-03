#include "htop.h"
#include "windows/htop.h"
#include "windows/top-bar.h"
#include <gtk/gtk.h>

#define _(name) htop_widget_ns_##name

static GtkWidget *_(widget);

static GtkWidget *_(init)(void) {
  _(widget) = gtk_button_new();
  gtk_widget_add_css_class(_(widget), "widget");
  gtk_widget_add_css_class(_(widget), "terminal");
  gtk_widget_add_css_class(_(widget), "padded");
  gtk_widget_add_css_class(_(widget), "clickable");
  gtk_widget_set_name(_(widget), "HTop");

  GtkWidget *label = gtk_label_new("Htop");
  gtk_button_set_child(GTK_BUTTON(_(widget)), label);

  return _(widget);
}

static void _(on_click)(void) {
  graphene_point_t bottom_right;
  if (!bottom_right_point_of(_(widget), TOP_BAR.window(), &bottom_right)) {
    fprintf(stderr, "Failed to compute bottom-right of the htop widget");
    return;
  }
  int margin_left = bottom_right.x - HTOP.width / 2.0;
  int margin_top = bottom_right.y;
  HTOP.move(margin_left, margin_top);

  HTOP.toggle();
}

static void _(activate)(void) {
  g_signal_connect(_(widget), "clicked", _(on_click), NULL);
}

widget_t HTOP_WIDGET = {.init = _(init), .activate = _(activate)};
