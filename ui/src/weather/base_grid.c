#include "ui/include/weather/base_grid.h"

G_DEFINE_TYPE(BaseGrid, base_grid, GTK_TYPE_GRID)

enum {
  PROP_0,
  PROP_COLS_COUNT,
  PROP_ROWS_COUNT,
};

static void base_grid_set_property(GObject *object, guint property_id,
                                   const GValue *value, GParamSpec *pspec) {
  BaseGrid *self = BASE_GRID(object);

  switch (property_id) {
  case PROP_COLS_COUNT:
    self->cols_count = g_value_get_uint(value);
    break;
  case PROP_ROWS_COUNT:
    self->rows_count = g_value_get_uint(value);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void base_grid_get_property(GObject *object, guint property_id,
                                   GValue *value, GParamSpec *pspec) {
  BaseGrid *self = BASE_GRID(object);

  switch (property_id) {
  case PROP_COLS_COUNT:
    g_value_set_uint(value, self->cols_count);
    break;
  case PROP_ROWS_COUNT:
    g_value_set_uint(value, self->rows_count);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void base_grid_class_init(BaseGridClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);

  object_class->set_property = base_grid_set_property;
  object_class->get_property = base_grid_get_property;

  g_object_class_install_property(object_class, PROP_COLS_COUNT,
                                  g_param_spec_uint("cols_count", "cols_count",
                                                    "cols_count", 0, 50, 0,
                                                    G_PARAM_READWRITE));

  g_object_class_install_property(object_class, PROP_ROWS_COUNT,
                                  g_param_spec_uint("rows_count", "rows_count",
                                                    "rows_count", 0, 50, 0,
                                                    G_PARAM_READWRITE));
}

static void base_grid_init(BaseGrid *self) {
  for (size_t col = 0; col < self->cols_count; col++) {
    gtk_grid_insert_column(GTK_GRID(self), col);
  }

  for (size_t row = 0; row < self->rows_count; row++) {
    gtk_grid_insert_row(GTK_GRID(self), row);
  }
}

GtkWidget *base_grid_new(size_t cols_count, size_t rows_count) {
  BaseGrid *self = g_object_new(BASE_GRID_TYPE,
                                //
                                "cols_count", cols_count,
                                //
                                "rows_count", rows_count,
                                //
                                NULL);

  return GTK_WIDGET(self);
}
