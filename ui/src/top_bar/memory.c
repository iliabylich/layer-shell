#include "ui/include/top_bar/memory.h"

struct _Memory {
  GtkButton parent_instance;
};

G_DEFINE_TYPE(Memory, memory, GTK_TYPE_BUTTON)

static void memory_class_init(MemoryClass *) {}

static void memory_init(Memory *self) {
  gtk_button_set_label(GTK_BUTTON(self), "--");
  gtk_widget_add_css_class(GTK_WIDGET(self), "widget");
  gtk_widget_add_css_class(GTK_WIDGET(self), "memory");
  gtk_widget_add_css_class(GTK_WIDGET(self), "padded");
  gtk_widget_add_css_class(GTK_WIDGET(self), "clickable");
  gtk_widget_set_cursor(GTK_WIDGET(self),
                        gdk_cursor_new_from_name("pointer", NULL));
  gtk_widget_set_name(GTK_WIDGET(self), "Memory");
}

GtkWidget *memory_new() { return g_object_new(memory_get_type(), NULL); }

void memory_refresh(Memory *memory, double used, double total) {
  char buffer[100];
  sprintf(buffer, "RAM %.1fG/%.1fG", used, total);
  gtk_button_set_label(GTK_BUTTON(memory), buffer);
}
