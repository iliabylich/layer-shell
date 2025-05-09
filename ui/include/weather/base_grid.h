#pragma once

#include <gtk/gtk.h>

struct _BaseGrid {
  GtkGrid parent_instance;

  size_t cols_count;
  size_t rows_count;
};

G_DECLARE_FINAL_TYPE(BaseGrid, base_grid, BASE_GRID, Widget, GtkGrid)

GtkWidget *base_grid_new(size_t cols_count, size_t rows_count);

#define TYPE_BASE_GRID base_grid_get_type()

#define BASE_GRID(obj)                                                         \
  (G_TYPE_CHECK_INSTANCE_CAST((obj), TYPE_BASE_GRID, BaseGrid))
