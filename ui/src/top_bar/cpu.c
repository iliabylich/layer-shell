#include "ui/include/top_bar/cpu.h"
#include "ui/include/builder.h"
#include "ui/include/top_bar/cpu_label.h"

typedef struct {
  CpuLabel **labels;
  size_t labels_count;
} data_t;
#define DATA_KEY "data"

GtkWidget *cpu_init() {
  GtkWidget *self = top_bar_get_widget("CPU");

  data_t *data = malloc(sizeof(data_t));
  data->labels = NULL;
  data->labels_count = 0;
  g_object_set_data_full(G_OBJECT(self), DATA_KEY, data, free);

  return self;
}

static bool first_time_init_p(data_t *data) { return data->labels_count == 0; }

static void assert_cpu_count_is(data_t *data, size_t count) {
  if (data->labels_count != count) {
    fprintf(stderr, "Dynamic number of CPU cores %lu vs %lu, exiting...\n",
            data->labels_count, count);
    exit(EXIT_FAILURE);
  }
}

static void create_labels(GtkWidget *self, size_t count) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);

  data->labels = calloc(count, sizeof(GtkWidget *));
  for (size_t i = 0; i < count; i++) {
    GtkWidget *label = cpu_label_new();
    data->labels[i] = CPU_LABEL(label);
    gtk_box_append(GTK_BOX(self), label);
  }
  data->labels_count = count;
}

void cpu_refresh(GtkWidget *self, IO_CArray_usize usage_per_core) {
  data_t *data = g_object_get_data(G_OBJECT(self), DATA_KEY);

  if (first_time_init_p(data)) {
    create_labels(self, usage_per_core.len);
  } else {
    assert_cpu_count_is(data, usage_per_core.len);
  }

  for (size_t i = 0; i < usage_per_core.len; i++) {
    cpu_label_set_load(data->labels[i], usage_per_core.ptr[i]);
  }
}
