#include "ui/include/top_bar/cpu.h"
#include "gtk/gtk.h"
#include "ui/include/top_bar/cpu_label.h"

struct _Cpu {
  GtkBox parent_instance;

  CpuLabel **labels;
  size_t labels_count;
};

G_DEFINE_TYPE(Cpu, cpu, GTK_TYPE_BOX)

static void cpu_class_init(CpuClass *) {}

static void cpu_init(Cpu *) {}

GtkWidget *cpu_new() {
  return g_object_new(CPU_TYPE,
                      //
                      "orientation", GTK_ORIENTATION_HORIZONTAL,
                      //
                      "spacing", 3,
                      //
                      "css-classes",
                      (const char *[]){"widget", "cpu", "padded", NULL},
                      //
                      "name", "CPU",
                      //
                      NULL);
}

static bool first_time_init_p(Cpu *self) { return self->labels_count == 0; }

static void assert_cpu_count_is(Cpu *self, size_t count) {
  if (self->labels_count != count) {
    fprintf(stderr, "Dynamic number of CPU cores %lu vs %lu, exiting...\n",
            self->labels_count, count);
    exit(EXIT_FAILURE);
  }
}

static void create_labels(Cpu *self, size_t count) {
  self->labels = calloc(count, sizeof(GtkWidget *));
  for (size_t i = 0; i < count; i++) {
    GtkWidget *label = cpu_label_new();
    self->labels[i] = CPU_LABEL(label);
    gtk_box_append(GTK_BOX(self), label);
  }
  self->labels_count = count;
}

void cpu_refresh(Cpu *self, IO_CArray_usize usage_per_core) {
  if (first_time_init_p(self)) {
    create_labels(self, usage_per_core.len);
  } else {
    assert_cpu_count_is(self, usage_per_core.len);
  }

  for (size_t i = 0; i < usage_per_core.len; i++) {
    cpu_label_set_load(self->labels[i], usage_per_core.ptr[i]);
  }
}
