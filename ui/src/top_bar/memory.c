#include "ui/include/top_bar/memory.h"
#include "gtk/gtk.h"

struct _Memory {
  GtkButton parent_instance;
};

G_DEFINE_TYPE(Memory, memory, GTK_TYPE_BUTTON)

static void memory_class_init(MemoryClass *) {}

static const char *css_classes[] = {"widget", "memory", "padded", "clickable",
                                    NULL};

static void memory_init(Memory *self) {
  gtk_widget_set_name(GTK_WIDGET(self), "Memory");
  gtk_widget_set_css_classes(GTK_WIDGET(self), css_classes);
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
