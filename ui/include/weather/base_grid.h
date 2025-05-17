#pragma once

#include <gtk/gtk.h>

typedef struct {
  size_t cols_count;
  size_t rows_count;
} base_grid_data_t;

void base_grid_init(GtkWidget *grid, size_t cols_count, size_t rows_count);
base_grid_data_t *base_grid_get_data(GtkWidget *grid);
