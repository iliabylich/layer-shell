#include "ui/include/top_bar/cpu_label.h"

GtkWidget *cpu_label_new() {
  GtkWidget *label = gtk_label_new("");
  gtk_label_set_use_markup(GTK_LABEL(label), true);
  cpu_label_set_load(label, 0);
  return label;
}

static const char *INDICATORS[] = {
    "<span color='#FFFFFF'>▁</span>", "<span color='#FFD5D5'>▂</span>",
    "<span color='#FFAAAA'>▃</span>", "<span color='#FF8080'>▄</span>",
    "<span color='#FF5555'>▅</span>", "<span color='#FF2B2B'>▆</span>",
    "<span color='#FF0000'>▇</span>", "<span color='#E60000'>█</span>",
};
static const size_t INDICATORS_COUNT =
    sizeof(INDICATORS) / sizeof(const char *);

void cpu_label_set_load(GtkWidget *label, float load) {
  size_t indicator_idx = floor(load / 100.0 * INDICATORS_COUNT);

  if (indicator_idx == INDICATORS_COUNT) {
    indicator_idx -= 1;
  }

  const char *markup = INDICATORS[indicator_idx];
  gtk_label_set_label(GTK_LABEL(label), markup);
}
