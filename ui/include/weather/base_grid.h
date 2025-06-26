#pragma once

#include <gtk/gtk.h>

void base_grid_init(GtkWidget *grid, size_t cols_count, size_t rows_count);
size_t base_grid_get_cols_count(GtkWidget *grid);
size_t base_grid_get_rows_count(GtkWidget *grid);
