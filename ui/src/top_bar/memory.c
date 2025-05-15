#include "ui/include/top_bar/memory.h"

struct _Memory {
  GtkButton parent_instance;
};

G_DEFINE_TYPE(Memory, memory, GTK_TYPE_BUTTON)

static void memory_class_init(MemoryClass *) {}

static void memory_init(Memory *) {}

GtkWidget *memory_new() {
  // clang-format off
  return g_object_new(
      MEMORY_TYPE,
      "name", "Memory",
      "css-classes", (const char *[]){"widget", "memory", "padded", "clickable", NULL},
      "cursor", gdk_cursor_new_from_name("pointer", NULL),
      "label", "--",
      NULL);
  // clang-format on
}

void memory_refresh(Memory *memory, double used, double total) {
  char buffer[100];
  sprintf(buffer, "RAM %.1fG/%.1fG", used, total);
  gtk_button_set_label(GTK_BUTTON(memory), buffer);
}
