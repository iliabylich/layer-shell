#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

GtkWidget *clock_init();
void clock_refresh(GtkWidget *clock, IO_ClockEvent event);
