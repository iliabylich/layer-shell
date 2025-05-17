#pragma once

#include "bindings.h"
#include <gtk/gtk.h>

typedef void (*launcher_exec_selected_f)();
typedef void (*launcher_go_up_f)();
typedef void (*launcher_go_down_f)();
typedef void (*launcher_reset_f)();
typedef void (*launcher_set_search_f)(const uint8_t *search);

GtkWidget *
launcher_init(GtkApplication *app,
              launcher_exec_selected_f launcher_exec_selected_callback,
              launcher_go_up_f launcher_go_up_callback,
              launcher_go_down_f launcher_go_down_callback,
              launcher_reset_f launcher_reset_callback,
              launcher_set_search_f launcher_set_search_callback);
void launcher_refresn(GtkWidget *launcher, IO_CArray_LauncherApp apps);
void launcher_toggle_and_reset(GtkWidget *launcher);
