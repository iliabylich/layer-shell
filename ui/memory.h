#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Memory, memory, MEMORY, WIDGET, GtkWidget)

#define MEMORY(obj) G_TYPE_CHECK_INSTANCE_CAST(obj, memory_get_type(), Memory)

GtkWidget *memory_new(void);
void memory_refresh(Memory *memory, IO_MemoryEvent event);
