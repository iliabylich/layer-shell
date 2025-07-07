#include "ui/cpu.h"
#include "ui/cpu_label.h"
#include "ui/logger.h"

LOGGER("CPU", 1)

struct _Cpu {
  GtkWidget parent_instance;

  GtkWidget *root;
  GList *labels;
  size_t labels_count;
};

G_DEFINE_TYPE(Cpu, cpu, GTK_TYPE_WIDGET)

static void cpu_init(Cpu *self) {
  LOG("init");

  self->labels = NULL;
  self->labels_count = 0;

  self->root = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 3);
  gtk_widget_add_css_class(self->root, "widget");
  gtk_widget_add_css_class(self->root, "cpu");
  gtk_widget_add_css_class(self->root, "padded");

  gtk_widget_set_parent(self->root, GTK_WIDGET(self));
}

static void cpu_dispose(GObject *object) {
  LOG("dispose");

  Cpu *self = CPU(object);
  g_clear_pointer(&self->root, gtk_widget_unparent);
  g_clear_pointer(&self->labels, g_list_free);
  G_OBJECT_CLASS(cpu_parent_class)->dispose(object);
}

static void cpu_class_init(CpuClass *klass) {
  LOG("class init");

  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->dispose = cpu_dispose;
  gtk_widget_class_set_layout_manager_type(GTK_WIDGET_CLASS(klass),
                                           GTK_TYPE_BOX_LAYOUT);
}

GtkWidget *cpu_new(void) { return g_object_new(cpu_get_type(), NULL); }

static bool first_time_init_p(Cpu *self) { return self->labels_count == 0; }

static void assert_cpu_count_is(size_t next, size_t prev) {
  if (next != prev) {
    fprintf(stderr, "Dynamic number of CPU cores %lu vs %lu, exiting...\n",
            next, prev);
    exit(EXIT_FAILURE);
  }
}

static void create_labels(Cpu *self, size_t count) {
  GList *labels = NULL;
  for (size_t i = 0; i < count; i++) {
    GtkWidget *label = cpu_label_new();
    labels = g_list_append(labels, label);
    gtk_box_append(GTK_BOX(self->root), label);
  }
  self->labels = labels;
  self->labels_count = count;
}

void cpu_refresh(Cpu *self, IO_CpuUsageEvent event) {
  if (first_time_init_p(self)) {
    create_labels(self, event.usage_per_core.len);
  } else {
    assert_cpu_count_is(self->labels_count, event.usage_per_core.len);
  }

  size_t i = 0;
  for (GList *ptr = self->labels; ptr != NULL; ptr = ptr->next) {
    GtkWidget *label = GTK_WIDGET(ptr->data);
    g_object_set(label, "load", event.usage_per_core.ptr[i], NULL);
    i++;
  }
}
