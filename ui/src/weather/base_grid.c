#include "ui/include/weather/base_grid.h"

typedef base_grid_data_t data_t;
#define DATA_KEY "data"

void base_grid_init(GtkWidget *self, size_t cols_count, size_t rows_count) {
  data_t *data = malloc(sizeof(data_t));
  data->cols_count = cols_count;
  data->rows_count = rows_count;
  g_object_set_data_full(G_OBJECT(self), DATA_KEY, data, free);

  for (size_t col = 0; col < cols_count; col++) {
    gtk_grid_insert_column(GTK_GRID(self), col);
  }

  for (size_t row = 0; row < rows_count; row++) {
    gtk_grid_insert_row(GTK_GRID(self), row);
  }
}

base_grid_data_t *base_grid_get_data(GtkWidget *self) {
  return g_object_get_data(G_OBJECT(self), DATA_KEY);
}
