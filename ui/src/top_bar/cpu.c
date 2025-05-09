#include "ui/include/top_bar/cpu.h"
#include "ui/include/top_bar/cpu_label.h"

struct _Cpu {
  GtkBox parent_instance;

  GtkLabel **labels;
  size_t labels_count;
};

G_DEFINE_TYPE(Cpu, cpu, GTK_TYPE_BOX)

static void cpu_class_init(CpuClass *) {}

static void cpu_init(Cpu *self) {
  gtk_orientable_set_orientation(GTK_ORIENTABLE(self),
                                 GTK_ORIENTATION_HORIZONTAL);
  gtk_box_set_spacing(GTK_BOX(self), 3);
  gtk_widget_add_css_class(GTK_WIDGET(self), "widget");
  gtk_widget_add_css_class(GTK_WIDGET(self), "cpu");
  gtk_widget_add_css_class(GTK_WIDGET(self), "padded");
  gtk_widget_set_name(GTK_WIDGET(self), "CPU");
}

GtkWidget *cpu_new() { return g_object_new(cpu_get_type(), NULL); }

static void create_labels_if_needed(Cpu *self, size_t count) {
  if (self->labels_count != 0) {
    if (self->labels_count == count) {
      return;
    } else {
      fprintf(stderr, "Dynamic number of CPU cores %lu vs %lu, exiting...\n",
              self->labels_count, count);
      exit(EXIT_FAILURE);
    }
  }

  self->labels_count = count;
  self->labels = calloc(count, sizeof(GtkWidget *));
  for (size_t i = 0; i < count; i++) {
    GtkLabel *label = cpu_label_new();
    self->labels[i] = label;
    gtk_box_append(GTK_BOX(self), GTK_WIDGET(label));
  }
}

void cpu_refresh(Cpu *self, IO_CArray_usize usage_per_core) {
  create_labels_if_needed(self, usage_per_core.len);

  for (size_t i = 0; i < usage_per_core.len; i++) {
    cpu_label_set_load(self->labels[i], usage_per_core.ptr[i]);
  }
}
