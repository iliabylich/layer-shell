#include "ui/include/weather/base_grid.h"
#include "ui/include/utils/has_prop.h"

WIDGET_HAS_PROP(rows_count, size_t)
WIDGET_HAS_PROP(cols_count, size_t)

void base_grid_init(GtkWidget *self, size_t cols_count, size_t rows_count) {
  set_cols_count(self, cols_count);
  set_rows_count(self, rows_count);

  for (size_t col = 0; col < cols_count; col++) {
    gtk_grid_insert_column(GTK_GRID(self), col);
  }

  for (size_t row = 0; row < rows_count; row++) {
    gtk_grid_insert_row(GTK_GRID(self), row);
  }
}

size_t base_grid_get_cols_count(GtkWidget *self) {
  return get_rows_count(self);
}
size_t base_grid_get_rows_count(GtkWidget *self) {
  return get_cols_count(self);
}
