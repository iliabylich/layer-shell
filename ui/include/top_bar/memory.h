#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

typedef void (*memory_clicked_f)();

GtkWidget *memory_init(memory_clicked_f callback);
void memory_refresh(GtkWidget *memory, IO_MemoryEvent event);
