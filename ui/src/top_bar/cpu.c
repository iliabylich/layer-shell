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

static const char *css_classes[] = {"widget", "cpu", "padded", NULL};

static void cpu_init(Cpu *self) {
  gtk_orientable_set_orientation(GTK_ORIENTABLE(self),
                                 GTK_ORIENTATION_HORIZONTAL);
  gtk_box_set_spacing(GTK_BOX(self), 3);
  gtk_widget_set_css_classes(GTK_WIDGET(self), css_classes);
  gtk_widget_set_name(GTK_WIDGET(self), "CPU");
}

GtkWidget *cpu_new() { return g_object_new(CPU_TYPE, NULL); }

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
