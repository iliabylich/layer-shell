#include "ui/include/top_bar/cpu.h"
#include "ui/include/builder.h"
#include "ui/include/top_bar/cpu_label.h"
#include "ui/include/utils/has_prop.h"

WIDGET_HAS_PROP(labels_list, CpuLabel **)
WIDGET_HAS_PROP(labels_count, size_t)

GtkWidget *cpu_init() {
  GtkWidget *self = top_bar_get_widget("CPU");
  set_labels_list(self, NULL);
  set_labels_count(self, 0);
  return self;
}

static bool first_time_init_p(GtkWidget *self) {
  return get_labels_count(self) == 0;
}

static void assert_cpu_count_is(size_t next, size_t prev) {
  if (next != prev) {
    fprintf(stderr, "Dynamic number of CPU cores %lu vs %lu, exiting...\n",
            next, prev);
    exit(EXIT_FAILURE);
  }
}

static void create_labels(GtkWidget *self, size_t count) {
  CpuLabel **labels = calloc(count, sizeof(GtkWidget *));
  for (size_t i = 0; i < count; i++) {
    GtkWidget *label = cpu_label_new();
    labels[i] = CPU_LABEL(label);
    gtk_box_append(GTK_BOX(self), label);
  }
  set_labels_list(self, labels);
  set_labels_count(self, count);
}

void cpu_refresh(GtkWidget *self, IO_CpuUsageEvent event) {
  if (first_time_init_p(self)) {
    create_labels(self, event.usage_per_core.len);
  } else {
    assert_cpu_count_is(get_labels_count(self), event.usage_per_core.len);
  }

  CpuLabel **labels = get_labels_list(self);

  for (size_t i = 0; i < event.usage_per_core.len; i++) {
    cpu_label_set_load(labels[i], event.usage_per_core.ptr[i]);
  }
}
