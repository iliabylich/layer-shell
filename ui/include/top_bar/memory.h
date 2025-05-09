#pragma once

#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Memory, memory, MEMORY, Widget, GtkButton)

GtkWidget *memory_new();
void memory_refresh(Memory *memory, double used, double total);

#define MEMORY(obj)                                                            \
  (G_TYPE_CHECK_INSTANCE_CAST((obj), memory_get_type(), Memory))
