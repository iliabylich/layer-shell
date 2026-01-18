#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

G_DECLARE_FINAL_TYPE(Clock, clock, CLOCK, WIDGET, GtkWidget)

#define CLOCK(obj) G_TYPE_CHECK_INSTANCE_CAST(obj, clock_get_type(), Clock)

GtkWidget *clock_new(void);
void clock_refresh(Clock *clock, IO_CString time);
