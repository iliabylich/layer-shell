#include "cpu.h"
#include "../bindings.h"
#include <gtk/gtk.h>

#define _(name) cpu_widget_ns_##name

static GtkWidget *_(widget);
static GtkWidget *_(labels)[12];
#define INDICATORS_COUNT 8
static const char *_(INDICATORS)[INDICATORS_COUNT] = {
    "<span color='#FFFFFF'>▁</span>", "<span color='#FFD5D5'>▂</span>",
    "<span color='#FFAAAA'>▃</span>", "<span color='#FF8080'>▄</span>",
    "<span color='#FF5555'>▅</span>", "<span color='#FF2B2B'>▆</span>",
    "<span color='#FF0000'>▇</span>", "<span color='#E60000'>█</span>",
};

static GtkWidget *_(init)(void) {
  _(widget) = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 3);
  gtk_widget_add_css_class(_(widget), "widget");
  gtk_widget_add_css_class(_(widget), "cpu");
  gtk_widget_add_css_class(_(widget), "padded");
  gtk_widget_set_name(_(widget), "CPU");

  for (size_t i = 0; i < 12; i++) {
    GtkWidget *label = gtk_label_new(NULL);
    gtk_label_set_use_markup(GTK_LABEL(label), true);
    gtk_box_append(GTK_BOX(_(widget)), label);
    _(labels)[i] = label;
  }

  return _(widget);
}

static void _(on_io_event)(const LAYER_SHELL_IO_Event *event) {
  switch (event->tag) {
  case CpuUsage: {
    for (size_t idx = 0; idx < 12; idx++) {
      GtkWidget *label = _(labels)[idx];
      size_t load = event->cpu_usage.usage_per_core.ptr[idx];
      size_t indicator_idx =
          (size_t)((double)load / 100.0 * (double)INDICATORS_COUNT);

      if (indicator_idx == INDICATORS_COUNT) {
        indicator_idx -= 1;
      }

      const char *markup = _(INDICATORS)[indicator_idx];
      gtk_label_set_label(GTK_LABEL(label), markup);
    }
    break;
  }
  default: {
    break;
  }
  }
}

static void _(activate)(void) { layer_shell_io_subscribe(_(on_io_event)); }

widget_t CPU_WIDGET = {.init = _(init), .activate = _(activate)};
