#include "ui/include/top_bar/memory.h"
#include "gtk/gtk.h"

struct _Memory {
  GtkButton parent_instance;
};

G_DEFINE_TYPE(Memory, memory, GTK_TYPE_BUTTON)

static void memory_class_init(MemoryClass *) {}

static void memory_init(Memory *self) {
  gtk_widget_set_name(GTK_WIDGET(self), "Memory");
  const char *CSS_CLASSES[] = {"widget", "memory", "padded", "clickable"};
  gtk_widget_set_css_classes(GTK_WIDGET(self), CSS_CLASSES);
  gtk_widget_set_cursor(GTK_WIDGET(self),
                        gdk_cursor_new_from_name("pointer", NULL));
  gtk_button_set_label(GTK_BUTTON(self), "--");
}

GtkWidget *memory_new() { return g_object_new(MEMORY_TYPE, NULL); }

void memory_refresh(Memory *memory, double used, double total) {
  char buffer[100];
  sprintf(buffer, "RAM %.1fG/%.1fG", used, total);
  gtk_button_set_label(GTK_BUTTON(memory), buffer);
}
